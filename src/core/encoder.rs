// Copyright (c) 2021, BlockProject 3D
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of BlockProject 3D nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

//! The BPX encoder.

use std::{
    collections::BTreeMap,
    io,
    io::{SeekFrom, Write}
};
use std::io::Seek;
use crate::core::compression::{Checksum, Crc32Checksum, Deflater, WeakChecksum, XzCompressionMethod, ZlibCompressionMethod};
use crate::core::header::{FLAG_CHECK_CRC32, FLAG_CHECK_WEAK, FLAG_COMPRESS_XZ, FLAG_COMPRESS_ZLIB, GetChecksum, MainHeader, SIZE_MAIN_HEADER, SIZE_SECTION_HEADER, Struct};
use crate::core::error::WriteError;
use crate::core::section::SectionEntry;
use crate::core::SectionData;
use crate::utils::ReadFill;

const READ_BLOCK_SIZE: usize = 8192;

fn write_sections<T: Write + Seek>(mut backend: T, sections: &mut BTreeMap<u32, SectionEntry>, file_start_offset: usize) -> Result<(u32, usize), WriteError>
{
    let mut ptr: u64 = file_start_offset as _;
    let mut all_sections_size: usize = 0;
    let mut chksum_sht: u32 = 0;

    for (idx, (_handle, section)) in sections.iter_mut().enumerate() {
        //At this point the handle must be valid otherwise sections_in_order is broken
        let data = section.data.as_mut().ok_or(WriteError::SectionNotLoaded)?;
        if data.size() > u32::MAX as usize {
            return Err(WriteError::Capacity(data.size()));
        }
        let last_section_ptr = data.stream_position()?;
        data.seek(io::SeekFrom::Start(0))?;
        let flags = section.entry1.get_flags(data.size() as u32);
        let (csize, chksum) = write_section(flags, data, &mut backend)?;
        data.seek(io::SeekFrom::Start(last_section_ptr))?;
        section.header.csize = csize as u32;
        section.header.size = data.size() as u32;
        section.header.chksum = chksum;
        section.header.flags = flags;
        section.header.pointer = ptr;
        section.index = idx as _;
        #[cfg(feature = "debug-log")]
        println!(
            "Writing section #{}: Size = {}, Size after compression = {}, Handle = {}",
            idx, section.header.size, section.header.csize, _handle
        );
        ptr += csize as u64;
        {
            //Locate section header offset, then directly write section header
            let header_start_offset = SIZE_MAIN_HEADER + (idx * SIZE_SECTION_HEADER);
            backend.seek(SeekFrom::Start(header_start_offset as _))?;
            section.header.write(&mut backend)?;
            //Reset file pointer back to the end of the last written section
            backend.seek(SeekFrom::Start(ptr))?;
        }
        chksum_sht += section.header.get_checksum();
        all_sections_size += csize;
    }
    Ok((chksum_sht, all_sections_size))
}

pub fn internal_save<T: Write + Seek>(mut backend: T, sections: &mut BTreeMap<u32, SectionEntry>, main_header: &mut MainHeader) -> Result<(), WriteError>
{
    let file_start_offset =
        SIZE_MAIN_HEADER + (SIZE_SECTION_HEADER * main_header.section_num as usize);
    //Seek to the start of the actual file content
    backend.seek(SeekFrom::Start(file_start_offset as _))?;
    //Write all section data and section headers
    let (chksum_sht, all_sections_size) = write_sections(&mut backend, sections, file_start_offset)?;
    main_header.file_size = all_sections_size as u64 + file_start_offset as u64;
    main_header.chksum = 0;
    main_header.chksum = chksum_sht + main_header.get_checksum();
    //Relocate to the start of the file and write the BPX main header
    backend.seek(SeekFrom::Start(0))?;
    main_header.write(&mut backend)?;
    Ok(())
}

fn write_last_section<T: Write + Seek>(mut backend: T, sections: &mut BTreeMap<u32, SectionEntry>, last_handle: u32) -> Result<(bool, i64), WriteError>
{
    let entry = sections.get_mut(&last_handle).unwrap();
    backend.seek(SeekFrom::Start(entry.header.pointer))?;
    let data = entry.data.as_mut().ok_or(WriteError::SectionNotLoaded)?;
    let last_section_ptr = data.stream_position()?;
    let flags = entry.entry1.get_flags(data.size() as u32);
    let (csize, chksum) = write_section(flags, data, &mut backend)?;
    data.seek(io::SeekFrom::Start(last_section_ptr))?;
    let old = entry.header;
    entry.header.csize = csize as u32;
    entry.header.size = data.size() as u32;
    entry.header.chksum = chksum;
    entry.header.flags = flags;
    let diff = entry.header.csize as i64 - old.csize as i64;
    Ok((old == entry.header, diff))
}

pub fn internal_save_last<T: Write + Seek>(mut backend: T, sections: &mut BTreeMap<u32, SectionEntry>, main_header: &mut MainHeader, last_handle: u32) -> Result<(), WriteError>
{
    // This function saves only the last section.
    let (update_sht, diff) = write_last_section(&mut backend, sections, last_handle)?;
    if update_sht {
        let offset_section_header = SIZE_MAIN_HEADER + (SIZE_SECTION_HEADER * (main_header.section_num - 1) as usize);
        backend.seek(SeekFrom::Start(offset_section_header as _))?;
        let entry = &sections[&last_handle];
        entry.header.write(&mut backend)?;
    }
    if diff != 0 {
        backend.seek(SeekFrom::Start(0))?;
        main_header.file_size = main_header.file_size.wrapping_add(diff as u64);
        main_header.write(&mut backend)?;
    }
    Ok(())
}

fn write_section_uncompressed<TWrite: Write, TChecksum: Checksum>(
    section: &mut dyn SectionData,
    out: &mut TWrite,
    chksum: &mut TChecksum
) -> Result<usize, WriteError>
{
    let mut idata: [u8; READ_BLOCK_SIZE] = [0; READ_BLOCK_SIZE];
    let mut count: usize = 0;
    while count < section.size() as usize {
        let res = section.read_fill(&mut idata)?;
        out.write_all(&idata[0..res])?;
        chksum.push(&idata[0..res]);
        count += res;
    }
    section.flush()?;
    Ok(section.size())
}

fn write_section_compressed<TMethod: Deflater, TWrite: Write, TChecksum: Checksum>(
    mut section: &mut dyn SectionData,
    out: &mut TWrite,
    chksum: &mut TChecksum
) -> Result<usize, WriteError>
{
    let size = section.size();
    let csize = TMethod::deflate(&mut section, out, size, chksum)?;
    Ok(csize)
}

fn write_section_checked<TWrite: Write, TChecksum: Checksum>(
    flags: u8,
    section: &mut dyn SectionData,
    out: &mut TWrite,
    chksum: &mut TChecksum
) -> Result<usize, WriteError>
{
    if flags & FLAG_COMPRESS_XZ != 0 {
        write_section_compressed::<XzCompressionMethod, _, _>(section, out, chksum)
    } else if flags & FLAG_COMPRESS_ZLIB != 0 {
        write_section_compressed::<ZlibCompressionMethod, _, _>(section, out, chksum)
    } else {
        write_section_uncompressed(section, out, chksum)
    }
}

pub fn write_section<TWrite: Write>(
    flags: u8,
    section: &mut dyn SectionData,
    out: &mut TWrite
) -> Result<(usize, u32), WriteError>
{
    if flags & FLAG_CHECK_CRC32 != 0 {
        let mut chksum = Crc32Checksum::new();
        let size = write_section_checked(flags, section, out, &mut chksum)?;
        Ok((size, chksum.finish()))
    } else if flags & FLAG_CHECK_WEAK != 0 {
        let mut chksum = WeakChecksum::new();
        let size = write_section_checked(flags, section, out, &mut chksum)?;
        Ok((size, chksum.finish()))
    } else {
        let mut chksum = WeakChecksum::new();
        let size = write_section_checked(flags, section, out, &mut chksum)?;
        Ok((size, 0))
    }
}
