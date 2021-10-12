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
    io,
    io::{Seek, SeekFrom, Write}
};

use crate::{
    compression::{Checksum, Crc32Checksum, Deflater, WeakChecksum, XzCompressionMethod, ZlibCompressionMethod},
    error::Error,
    header::{
        MainHeader,
        SectionHeader,
        FLAG_CHECK_CRC32,
        FLAG_CHECK_WEAK,
        FLAG_COMPRESS_XZ,
        FLAG_COMPRESS_ZLIB,
        SIZE_MAIN_HEADER,
        SIZE_SECTION_HEADER
    },
    section::{new_section_data, SectionData},
    Interface,
    Result,
    SectionHandle
};

const READ_BLOCK_SIZE: usize = 8192;

/// Represents the IO backend for a BPX encoder.
pub trait IoBackend: io::Write + io::Seek
{
}
impl<T: io::Write + io::Seek> IoBackend for T {}

struct SectionEntry
{
    header: SectionHeader,
    data: Box<dyn SectionData>,
    index: u32
}

/// The BPX encoder.
pub struct Encoder<TBackend: IoBackend>
{
    main_header: MainHeader,
    sections: Vec<Option<SectionEntry>>,
    sections_in_order: Vec<SectionHandle>,
    file: TBackend,
    cur_index: u32,
    modified: bool
}

impl<TBackend: IoBackend> Encoder<TBackend>
{
    /// Creates a new BPX encoder.
    ///
    /// # Arguments
    ///
    /// * `file`: An [IoBackend](self::IoBackend) to use for reading the data.
    ///
    /// returns: Result<Encoder<TBackend>, Error>
    pub fn new(file: TBackend) -> Result<Encoder<TBackend>>
    {
        return Ok(Encoder {
            main_header: MainHeader::new(),
            sections: Vec::new(),
            sections_in_order: Vec::new(),
            file,
            cur_index: 0,
            modified: true
        });
    }

    /// Sets the BPX Main Header.
    ///
    /// # Arguments
    ///
    /// * `main_header`: the new [MainHeader](crate::header::MainHeader).
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::builder::MainHeaderBuilder;
    /// use bpx::encoder::Encoder;
    /// use bpx::Interface;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut encoder = Encoder::new(new_byte_buf(0)).unwrap();
    /// encoder.set_main_header(MainHeaderBuilder::new().with_type(1).build());
    /// assert_eq!(encoder.get_main_header().btype, 1);
    /// ```
    pub fn set_main_header(&mut self, main_header: MainHeader)
    {
        self.main_header = main_header;
    }

    /// Creates a new section in the BPX
    ///
    /// # Arguments
    ///
    /// * `header`: the [SectionHeader](crate::header::SectionHeader) of the new section.
    ///
    /// returns: Result<SectionHandle, Error>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::builder::MainHeaderBuilder;
    /// use bpx::encoder::Encoder;
    /// use bpx::header::SectionHeader;
    /// use bpx::Interface;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut encoder = Encoder::new(new_byte_buf(0)).unwrap();
    /// assert_eq!(encoder.get_main_header().section_num, 0);
    /// encoder.create_section(SectionHeader::new());
    /// assert_eq!(encoder.get_main_header().section_num, 1);
    /// ```
    pub fn create_section(&mut self, header: SectionHeader) -> Result<SectionHandle>
    {
        self.modified = true;
        self.main_header.section_num += 1;
        let section = create_section(&header)?;
        for i in 0..self.sections.len() {
            if self.sections[i].is_none() {
                self.sections[i] = Some(SectionEntry {
                    header,
                    data: section,
                    index: self.cur_index
                });
                self.cur_index += 1;
                self.sections_in_order.push(SectionHandle(i));
                return Ok(SectionHandle(i));
            }
        }
        self.sections.push(Some(SectionEntry {
            header,
            data: section,
            index: self.cur_index
        }));
        self.cur_index += 1;
        let r = self.sections.len() - 1;
        self.sections_in_order.push(SectionHandle(r));
        return Ok(SectionHandle(r));
    }

    fn write_sections(&mut self, file_start_offset: usize) -> Result<(u32, usize)>
    {
        let mut ptr: u64 = file_start_offset as _;
        let mut all_sections_size: usize = 0;
        let mut chksum_sht: u32 = 0;

        for v in &self.sections_in_order {
            //At this point the handle must be valid otherwise sections_in_order is broken
            let section = self.sections[v.0].as_mut().unwrap();
            if section.data.size() > u32::MAX as usize {
                return Err(Error::Capacity(section.data.size()));
            }
            section.data.seek(io::SeekFrom::Start(0))?;
            let flags = get_flags(&section.header, section.data.size() as u32);
            let (csize, chksum) = write_section(flags, section.data.as_mut(), &mut self.file)?;
            section.header.csize = csize as u32;
            section.header.size = section.data.size() as u32;
            section.header.chksum = chksum;
            section.header.flags = flags;
            section.header.pointer = ptr;
            #[cfg(feature = "debug-log")]
            println!(
                "Writing section #{}: Size = {}, Size after compression = {}",
                section.index, section.header.size, section.header.csize
            );
            ptr += csize as u64;
            {
                //Locate section header offset, then directly write section header
                let header_start_offset = SIZE_MAIN_HEADER + (section.index as usize * SIZE_SECTION_HEADER);
                self.file.seek(SeekFrom::Start(header_start_offset as _))?;
                section.header.write(&mut self.file)?;
                //Reset file pointer back to the end of the last written section
                self.file.seek(SeekFrom::Start(ptr))?;
            }
            chksum_sht += section.header.get_checksum();
            all_sections_size += csize;
        }
        return Ok((chksum_sht, all_sections_size));
    }

    /// Writes all sections to the underlying IO backend.
    ///
    /// **This function prints some information to standard output as a way
    /// to debug data compression issues unless the `debug-log` feature
    /// is disabled.**
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned if some data could
    /// not be written.
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::encoder::Encoder;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut encoder = Encoder::new(new_byte_buf(0)).unwrap();
    /// encoder.save();
    /// //TODO: Finish once Encoder can be consumed back into its IO Backend
    /// ```
    pub fn save(&mut self) -> Result<()>
    {
        if !self.modified { //If the file has not been modified do not engage any IO
            return Ok(());
        }
        let file_start_offset = SIZE_MAIN_HEADER + (SIZE_SECTION_HEADER * self.main_header.section_num as usize);
        //Seek to the start of the actual file content
        self.file.seek(SeekFrom::Start(file_start_offset as _))?;
        //Write all section data and section headers
        let (chksum_sht, all_sections_size) = self.write_sections(file_start_offset)?;
        self.main_header.file_size = all_sections_size as u64 + file_start_offset as u64;
        self.main_header.chksum = chksum_sht + self.main_header.get_checksum();
        //Relocate to the start of the file and write the BPX main header
        self.file.seek(SeekFrom::Start(0))?;
        self.main_header.write(&mut self.file)?;
        self.modified = false;
        return Ok(());
    }
}

impl<TBackend: IoBackend> Interface for Encoder<TBackend>
{
    fn find_section_by_type(&self, btype: u8) -> Option<SectionHandle>
    {
        for i in 0..self.sections.len() {
            if let Some(v) = &self.sections[i] {
                if v.header.btype == btype {
                    return Some(SectionHandle(i));
                }
            }
        }
        return None;
    }

    fn find_all_sections_of_type(&self, btype: u8) -> Vec<SectionHandle>
    {
        let mut v = Vec::new();

        for i in 0..self.sections.len() {
            if let Some(vv) = &self.sections[i] {
                if vv.header.btype == btype {
                    v.push(SectionHandle(i));
                }
            }
        }
        return v;
    }

    fn find_section_by_index(&self, index: u32) -> Option<SectionHandle>
    {
        if let Some(s) = self.sections_in_order.get(index as usize) {
            return Some(*s);
        }
        return None;
    }

    fn get_section_header(&self, handle: SectionHandle) -> &SectionHeader
    {
        return &self.sections[handle.0].as_ref().unwrap().header;
    }

    fn get_section_index(&self, handle: SectionHandle) -> u32
    {
        return handle.0 as u32;
    }

    fn open_section(&mut self, handle: SectionHandle) -> Result<&mut dyn SectionData>
    {
        let section = self.sections[handle.0].as_mut().unwrap();
        self.modified = true;
        return Ok(section.data.as_mut());
    }

    fn get_main_header(&self) -> &MainHeader
    {
        return &self.main_header;
    }
}

fn get_flags(header: &SectionHeader, size: u32) -> u8
{
    let mut flags = 0;
    if header.flags & FLAG_CHECK_WEAK != 0 {
        flags |= FLAG_CHECK_WEAK;
    } else if header.flags & FLAG_CHECK_CRC32 != 0 {
        flags |= FLAG_CHECK_CRC32;
    }
    if header.flags & FLAG_COMPRESS_XZ != 0 && size > header.csize {
        flags |= FLAG_COMPRESS_XZ;
    } else if header.flags & FLAG_COMPRESS_ZLIB != 0 && size > header.csize {
        flags |= FLAG_COMPRESS_ZLIB;
    }
    return flags;
}

fn create_section(header: &SectionHeader) -> Result<Box<dyn SectionData>>
{
    if header.size == 0 {
        let mut section = new_section_data(None)?;
        section.seek(io::SeekFrom::Start(0))?;
        return Ok(section);
    } else {
        let mut section = new_section_data(Some(header.size))?;
        section.seek(io::SeekFrom::Start(0))?;
        return Ok(section);
    }
}

fn write_section_uncompressed<TWrite: Write, TChecksum: Checksum>(
    section: &mut dyn SectionData,
    out: &mut TWrite,
    chksum: &mut TChecksum
) -> Result<usize>
{
    let mut idata: [u8; READ_BLOCK_SIZE] = [0; READ_BLOCK_SIZE];
    let mut count: usize = 0;
    while count < section.size() as usize {
        let res = section.read(&mut idata)?;
        out.write(&idata[0..res])?;
        chksum.push(&idata[0..res]);
        count += res;
    }
    section.flush()?;
    return Ok(section.size());
}

fn write_section_compressed<TMethod: Deflater, TWrite: Write, TChecksum: Checksum>(
    mut section: &mut dyn SectionData,
    out: &mut TWrite,
    chksum: &mut TChecksum
) -> Result<usize>
{
    let size = section.size();
    let csize = TMethod::deflate(&mut section, out, size, chksum)?;
    return Ok(csize);
}

fn write_section_checked<TWrite: Write, TChecksum: Checksum>(
    flags: u8,
    section: &mut dyn SectionData,
    out: &mut TWrite,
    chksum: &mut TChecksum
) -> Result<usize>
{
    if flags & FLAG_COMPRESS_XZ != 0 {
        return write_section_compressed::<XzCompressionMethod, _, _>(section, out, chksum);
    } else if flags & FLAG_COMPRESS_ZLIB != 0 {
        return write_section_compressed::<ZlibCompressionMethod, _, _>(section, out, chksum);
    } else {
        return write_section_uncompressed(section, out, chksum);
    }
}

fn write_section<TWrite: Write>(flags: u8, section: &mut dyn SectionData, out: &mut TWrite) -> Result<(usize, u32)>
{
    if flags & FLAG_CHECK_CRC32 != 0 {
        let mut chksum = Crc32Checksum::new();
        let size = write_section_checked(flags, section, out, &mut chksum)?;
        return Ok((size, chksum.finish()));
    } else if flags & FLAG_CHECK_WEAK != 0 {
        let mut chksum = WeakChecksum::new();
        let size = write_section_checked(flags, section, out, &mut chksum)?;
        return Ok((size, chksum.finish()));
    } else {
        let mut chksum = WeakChecksum::new();
        let size = write_section_checked(flags, section, out, &mut chksum)?;
        return Ok((size, 0));
    }
}
