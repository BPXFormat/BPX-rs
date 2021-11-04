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
    io::{SeekFrom, Write},
    ops::Bound,
    rc::Rc
};

use crate::{
    compression::{
        Checksum,
        Crc32Checksum,
        Deflater,
        WeakChecksum,
        XzCompressionMethod,
        ZlibCompressionMethod
    },
    error::WriteError,
    header::{
        GetChecksum,
        MainHeader,
        SectionHeader,
        Struct,
        FLAG_CHECK_CRC32,
        FLAG_CHECK_WEAK,
        FLAG_COMPRESS_XZ,
        FLAG_COMPRESS_ZLIB,
        SIZE_MAIN_HEADER,
        SIZE_SECTION_HEADER
    },
    section::{AutoSection, Section, SectionData},
    Handle,
    Interface
};
use crate::utils::ReadFill;

const READ_BLOCK_SIZE: usize = 8192;

/// Represents the IO backend for a BPX encoder.
pub trait IoBackend: io::Write + io::Seek
{
}
impl<T: io::Write + io::Seek> IoBackend for T {}

struct SectionEntry
{
    header: SectionHeader,
    data: Rc<AutoSection>,
    threshold: u32,
    flags: u8
}

/// The BPX encoder.
pub struct Encoder<TBackend: IoBackend>
{
    main_header: MainHeader,
    sections: BTreeMap<u32, SectionEntry>,
    file: TBackend,
    next_handle: u32,
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
    pub fn new(file: TBackend) -> Result<Encoder<TBackend>, WriteError>
    {
        return Ok(Encoder {
            main_header: MainHeader::new(),
            sections: BTreeMap::new(),
            file,
            next_handle: 0,
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
        self.modified = true;
    }

    /// Creates a new section in the BPX
    ///
    /// # Arguments
    ///
    /// * `header`: the [SectionHeader](crate::header::SectionHeader) of the new section.
    ///
    /// returns: Result<Handle, Error>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::builder::MainHeaderBuilder;
    /// use bpx::encoder::Encoder;
    /// use bpx::header::{SectionHeader, Struct};
    /// use bpx::Interface;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut encoder = Encoder::new(new_byte_buf(0)).unwrap();
    /// assert_eq!(encoder.get_main_header().section_num, 0);
    /// encoder.create_section(SectionHeader::new());
    /// assert_eq!(encoder.get_main_header().section_num, 1);
    /// ```
    pub fn create_section(&mut self, header: SectionHeader)
        -> Result<&Rc<AutoSection>, WriteError>
    {
        self.modified = true;
        self.main_header.section_num += 1;
        let r = self.next_handle;
        let section = create_section(&header, Handle(r))?;
        let entry = SectionEntry {
            header,
            data: section,
            threshold: header.csize,
            flags: header.flags
        };
        self.sections.insert(r, entry);
        self.next_handle += 1;
        return Ok(&self.sections[&r].data);
    }

    /// Removes a section from this BPX.
    ///
    /// # Panics
    ///
    /// Panics if the given section handle is invalid.
    ///
    /// # Arguments
    ///
    /// * `handle`: a handle to the section.
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::encoder::Encoder;
    /// use bpx::header::{SectionHeader, Struct};
    /// use bpx::Interface;
    /// use bpx::section::Section;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut encoder = Encoder::new(new_byte_buf(0)).unwrap();
    /// let section = encoder.create_section(SectionHeader::new()).unwrap().clone();
    /// encoder.save();
    /// assert_eq!(encoder.get_main_header().section_num, 1);
    /// encoder.remove_section(section.handle());
    /// encoder.save();
    /// assert_eq!(encoder.get_main_header().section_num, 0);
    /// ```
    pub fn remove_section(&mut self, handle: Handle)
    {
        self.sections.remove(&handle.0);
        self.main_header.section_num -= 1;
        self.modified = true;
    }

    fn write_sections(&mut self, file_start_offset: usize) -> Result<(u32, usize), WriteError>
    {
        let mut ptr: u64 = file_start_offset as _;
        let mut all_sections_size: usize = 0;
        let mut chksum_sht: u32 = 0;

        for (idx, (_handle, section)) in self.sections.iter_mut().enumerate() {
            //At this point the handle must be valid otherwise sections_in_order is broken
            if section.data.size() > u32::MAX as usize {
                return Err(WriteError::Capacity(section.data.size()));
            }
            let mut data = section.data.open()?;
            let last_section_ptr = data.stream_position()?;
            data.seek(io::SeekFrom::Start(0))?;
            let flags = get_flags(section, data.size() as u32);
            let (csize, chksum) = write_section(flags, data.as_mut(), &mut self.file)?;
            data.seek(io::SeekFrom::Start(last_section_ptr))?;
            section.header.csize = csize as u32;
            section.header.size = data.size() as u32;
            section.header.chksum = chksum;
            section.header.flags = flags;
            section.header.pointer = ptr;
            #[cfg(feature = "debug-log")]
            println!(
                "Writing section #{}: Size = {}, Size after compression = {}, Handle = {}",
                idx, section.header.size, section.header.csize, _handle
            );
            ptr += csize as u64;
            {
                //Locate section header offset, then directly write section header
                let header_start_offset = SIZE_MAIN_HEADER + (idx * SIZE_SECTION_HEADER);
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

    fn internal_save(&mut self) -> Result<(), WriteError>
    {
        let file_start_offset =
            SIZE_MAIN_HEADER + (SIZE_SECTION_HEADER * self.main_header.section_num as usize);
        //Seek to the start of the actual file content
        self.file.seek(SeekFrom::Start(file_start_offset as _))?;
        //Write all section data and section headers
        let (chksum_sht, all_sections_size) = self.write_sections(file_start_offset)?;
        self.main_header.file_size = all_sections_size as u64 + file_start_offset as u64;
        self.main_header.chksum = 0;
        self.main_header.chksum = chksum_sht + self.main_header.get_checksum();
        //Relocate to the start of the file and write the BPX main header
        self.file.seek(SeekFrom::Start(0))?;
        self.main_header.write(&mut self.file)?;
        self.modified = false;
        return Ok(());
    }

    fn write_last_section(&mut self, last_handle: u32) -> Result<(bool, i64), WriteError>
    {
        let entry = self.sections.get_mut(&last_handle).unwrap();
        self.file.seek(SeekFrom::Start(entry.header.pointer))?;
        let mut data = entry.data.open()?;
        let last_section_ptr = data.stream_position()?;
        let flags = get_flags(entry, data.size() as u32);
        let (csize, chksum) = write_section(flags, data.as_mut(), &mut self.file)?;
        data.seek(io::SeekFrom::Start(last_section_ptr))?;
        let old = entry.header;
        entry.header.csize = csize as u32;
        entry.header.size = data.size() as u32;
        entry.header.chksum = chksum;
        entry.header.flags = flags;
        let diff = entry.header.csize as i64 - old.csize as i64;
        return Ok((old == entry.header, diff));
    }

    fn internal_save_last(&mut self) -> Result<(), WriteError>
    {
        // This function saves only the last section.
        let (update_sht, diff) = self.write_last_section(self.next_handle - 1)?;
        if update_sht {
            let offset_section_header = SIZE_MAIN_HEADER
                + (SIZE_SECTION_HEADER * (self.main_header.section_num - 1) as usize);
            self.file
                .seek(SeekFrom::Start(offset_section_header as _))?;
            let entry = &self.sections[&(self.next_handle - 1)];
            entry.header.write(&mut self.file)?;
        }
        if diff != 0 {
            self.file.seek(SeekFrom::Start(0))?;
            self.main_header.file_size = self.main_header.file_size.wrapping_add(diff as u64);
            self.main_header.write(&mut self.file)?;
        }
        return Ok(());
    }

    /// Writes all sections to the underlying IO backend.
    ///
    /// **This function prints some information to standard output as a way
    /// to debug data compression issues unless the `debug-log` feature
    /// is disabled.**
    ///
    /// # Errors
    ///
    /// A [WriteError](crate::error::WriteError) is returned if some data could
    /// not be written.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Seek, SeekFrom};
    /// use bpx::decoder::Decoder;
    /// use bpx::encoder::Encoder;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut encoder = Encoder::new(new_byte_buf(0)).unwrap();
    /// encoder.save();
    /// let mut bytebuf = encoder.into_inner();
    /// bytebuf.seek(SeekFrom::Start(0)).unwrap();
    /// Decoder::new(bytebuf).unwrap(); //If this panics then encoder is broken
    /// ```
    pub fn save(&mut self) -> Result<(), WriteError>
    {
        let mut filter = self
            .sections
            .iter()
            .filter(|(_, entry)| entry.data.modified());
        let count = filter.by_ref().count();
        if self.modified || count > 1 {
            self.modified = false;
            return self.internal_save();
        } else if !self.modified && count == 1 {
            let (handle, _) = filter.last().unwrap();
            if *handle == self.next_handle - 1 {
                //Save only the last section (no need to re-write every other section
                return self.internal_save_last();
            } else {
                //Unfortunately the modified section is not the last one so we can't safely
                //expand/reduce the file size without corrupting other sections
                return self.internal_save();
            }
        }
        return Ok(());
    }

    /// Consumes this BPX encoder and returns the inner IO backend.
    pub fn into_inner(self) -> TBackend
    {
        return self.file;
    }
}

impl<TBackend: IoBackend> Interface for Encoder<TBackend>
{
    fn find_section_by_type(&self, btype: u8) -> Option<Handle>
    {
        for (handle, entry) in &self.sections {
            if entry.header.btype == btype {
                return Some(Handle(*handle));
            }
        }
        return None;
    }

    fn find_all_sections_of_type(&self, btype: u8) -> Vec<Handle>
    {
        let mut v = Vec::new();

        for (handle, entry) in &self.sections {
            if entry.header.btype == btype {
                v.push(Handle(*handle));
            }
        }
        return v;
    }

    fn find_section_by_index(&self, index: u32) -> Option<Handle>
    {
        for (idx, handle) in self.sections.keys().enumerate() {
            if idx as u32 == index {
                return Some(Handle(*handle));
            }
        }
        return None;
    }

    fn get_section_header(&self, handle: Handle) -> &SectionHeader
    {
        return &self.sections[&handle.0].header;
    }

    fn get_section_index(&self, handle: Handle) -> u32
    {
        return self
            .sections
            .range((Bound::Unbounded, Bound::Excluded(handle.0)))
            .count() as u32;
    }

    fn get_section(&self, handle: Handle) -> &Rc<AutoSection>
    {
        return &self.sections[&handle.0].data;
    }

    fn get_main_header(&self) -> &MainHeader
    {
        return &self.main_header;
    }
}

fn get_flags(section: &SectionEntry, size: u32) -> u8
{
    let mut flags = 0;
    if section.flags & FLAG_CHECK_WEAK != 0 {
        flags |= FLAG_CHECK_WEAK;
    } else if section.flags & FLAG_CHECK_CRC32 != 0 {
        flags |= FLAG_CHECK_CRC32;
    }
    if section.flags & FLAG_COMPRESS_XZ != 0 && size > section.threshold {
        flags |= FLAG_COMPRESS_XZ;
    } else if section.flags & FLAG_COMPRESS_ZLIB != 0 && size > section.threshold {
        flags |= FLAG_COMPRESS_ZLIB;
    }
    return flags;
}

fn create_section(header: &SectionHeader, handle: Handle) -> Result<Rc<AutoSection>, WriteError>
{
    let section = Rc::new(AutoSection::new(header.size, handle)?);
    {
        let mut data = section.open().unwrap();
        data.seek(io::SeekFrom::Start(0))?;
    } //Another defect of the Rust borrow checker
    return Ok(section);
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
    return Ok(section.size());
}

fn write_section_compressed<TMethod: Deflater, TWrite: Write, TChecksum: Checksum>(
    mut section: &mut dyn SectionData,
    out: &mut TWrite,
    chksum: &mut TChecksum
) -> Result<usize, WriteError>
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
) -> Result<usize, WriteError>
{
    if flags & FLAG_COMPRESS_XZ != 0 {
        return write_section_compressed::<XzCompressionMethod, _, _>(section, out, chksum);
    } else if flags & FLAG_COMPRESS_ZLIB != 0 {
        return write_section_compressed::<ZlibCompressionMethod, _, _>(section, out, chksum);
    } else {
        return write_section_uncompressed(section, out, chksum);
    }
}

fn write_section<TWrite: Write>(
    flags: u8,
    section: &mut dyn SectionData,
    out: &mut TWrite
) -> Result<(usize, u32), WriteError>
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
