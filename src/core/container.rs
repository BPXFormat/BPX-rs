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

use std::io;
use std::collections::{Bound, BTreeMap};
use std::io::{Read, Seek, SeekFrom};
use super::decoder::load_section1;
use super::encoder::write_section;
use crate::core::error::{ReadError, WriteError};
use crate::Handle;
use crate::core::header::{FLAG_CHECK_CRC32, FLAG_CHECK_WEAK, FLAG_COMPRESS_XZ, FLAG_COMPRESS_ZLIB, GetChecksum, MainHeader, SectionHeader, SIZE_MAIN_HEADER, SIZE_SECTION_HEADER, Struct};
use crate::section::{new_section_data, SectionData};
use crate::utils::OptionExtension;

pub const DEFAULT_COMPRESSION_THRESHOLD: u32 = 65536;

pub struct IterMut<'a, T>
{
    backend: &'a mut T,
    sections: std::collections::btree_map::IterMut<'a, u32, SectionEntry>
}

impl<'a, T> Iterator for IterMut<'a, T>
{
    type Item = SectionMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item>
    {
        let (h, v) = self.sections.next()?;
        unsafe {
            let ptr = self.backend as *mut T;
            Some(SectionMut {
                backend: &mut *ptr,
                entry: v,
                handle: Handle(*h)
            })
        }
    }
}

pub struct SectionMut<'a, T>
{
    backend: &'a mut T,
    entry: &'a mut SectionEntry,
    handle: Handle
}

impl<'a, T: Read + Seek> SectionMut<'a, T>
{
    pub fn load(&mut self) -> Result<&mut dyn SectionData, ReadError>
    {
        let data = self.entry.data.get_or_insert_with_err(|| load_section1(self.backend, &self.entry.header))?;
        self.entry.modified = true;
        Ok(&mut **data)
    }
}

impl<'a, T> SectionMut<'a, T>
{
    pub fn open(&mut self) -> Option<&mut dyn SectionData>
    {
        self.entry.modified = true;
        match &mut self.entry.data {
            Some(v) => Some(&mut **v),
            None => None
        }
    }

    pub fn handle(&self) -> Handle
    {
        self.handle
    }

    pub fn header(&self) -> &SectionHeader
    {
        &self.entry.header
    }

    pub fn size(&self) -> usize
    {
        self.entry.data.as_ref().map(|v| v.size()).unwrap_or(0)
    }

    pub fn index(&self) -> u32
    {
        self.entry.index
    }
}

pub struct Section<'a>
{
    entry: &'a SectionEntry,
    handle: Handle
}

impl<'a> Section<'a>
{
    pub fn size(&self) -> usize
    {
        self.entry.data.as_ref().map(|v| v.size()).unwrap_or(0)
    }

    pub fn handle(&self) -> Handle
    {
        self.handle
    }

    pub fn header(&self) -> &SectionHeader
    {
        &self.entry.header
    }

    pub fn index(&self) -> u32
    {
        self.entry.index
    }
}

struct SectionEntry1
{
    threshold: u32,
    flags: u8,
}

impl SectionEntry1
{
    pub fn get_flags(&self, size: u32) -> u8
    {
        let mut flags = 0;
        if self.flags & FLAG_CHECK_WEAK != 0 {
            flags |= FLAG_CHECK_WEAK;
        } else if self.flags & FLAG_CHECK_CRC32 != 0 {
            flags |= FLAG_CHECK_CRC32;
        }
        if self.flags & FLAG_COMPRESS_XZ != 0 && size > self.threshold {
            flags |= FLAG_COMPRESS_XZ;
        } else if self.flags & FLAG_COMPRESS_ZLIB != 0 && size > self.threshold {
            flags |= FLAG_COMPRESS_ZLIB;
        }
        flags
    }
}

struct SectionEntry
{
    entry1: SectionEntry1,
    header: SectionHeader,
    data: Option<Box<dyn SectionData>>,
    index: u32,
    modified: bool
}

pub struct Container<T>
{
    backend: T,
    main_header: MainHeader,
    sections: BTreeMap<u32, SectionEntry>,
    next_handle: u32,
    modified: bool
}

impl<T> Container<T>
{
    pub fn find_section_by_type(&self, btype: u8) -> Option<Handle>
    {
        for (handle, entry) in &self.sections {
            if entry.header.btype == btype {
                return Some(Handle(*handle));
            }
        }
        None
    }

    pub fn find_section_by_index(&self, index: u32) -> Option<Handle>
    {
        for (idx, handle) in self.sections.keys().enumerate() {
            if idx as u32 == index {
                return Some(Handle(*handle));
            }
        }
        None
    }

    /// Sets the BPX Main Header.
    ///
    /// # Arguments
    ///
    /// * `main_header`: the new [MainHeader](crate::header::MainHeader).
    pub fn set_main_header<H: Into<MainHeader>>(&mut self, main_header: H)
    {
        self.main_header = main_header.into();
        self.modified = true;
    }

    pub fn get_main_header(&self) -> &MainHeader
    {
        &self.main_header
    }

    pub fn get(&self, handle: Handle) -> Section
    {
        self.sections.get(&handle.0).map(|v| Section {
            handle,
            entry: v
        }).expect("attempt to use invalid handle")
    }

    pub fn get_mut(&mut self, handle: Handle) -> SectionMut<T>
    {
        self.sections.get_mut(&handle.0).map(|v| SectionMut {
            handle,
            entry: v,
            backend: &mut self.backend
        }).expect("attempt to use invalid handle")
    }

    /// Creates a new section in the BPX
    ///
    /// # Arguments
    ///
    /// * `header`: the [SectionHeader](crate::header::SectionHeader) of the new section.
    ///
    /// returns: Result<Handle, Error>
    pub fn create_section<H: Into<SectionHeader>>(&mut self, header: H) -> Result<Handle, WriteError>
    {
        self.modified = true;
        self.main_header.section_num += 1;
        let r = self.next_handle;
        let section = new_section_data(None)?;
        let h = header.into();
        let entry = SectionEntry {
            header: h,
            data: Some(section),
            modified: false,
            index: self.main_header.section_num - 1,
            entry1: SectionEntry1 {
                threshold: h.csize,
                flags: h.flags
            }
        };
        self.sections.insert(r, entry);
        self.next_handle += 1;
        Ok(Handle(r))
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
    pub fn remove_section(&mut self, handle: Handle)
    {
        self.sections.remove(&handle.0);
        self.main_header.section_num -= 1;
        self.modified = true;
        self.sections.range_mut((Bound::Included(handle.0), Bound::Unbounded)).for_each(|(_, v)| {
            v.index -= 1;
        });
    }

    pub fn iter(&self) -> impl Iterator<Item=Section>
    {
        self.sections.iter().map(|(h, v)| Section {
            handle: Handle(*h),
            entry: v
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item=SectionMut<T>>
    {
        IterMut {
            backend: &mut self.backend,
            sections: self.sections.iter_mut()
        }
    }

    /// Consumes this BPX container and returns the inner IO backend.
    pub fn into_inner(self) -> T
    {
        self.backend
    }
}

impl<T: io::Read + io::Seek> Container<T>
{
    fn read_section_header_table(&mut self, checksum: u32) -> Result<(), ReadError>
    {
        let mut final_checksum = checksum;

        for i in 0..self.main_header.section_num {
            let (checksum, header) = SectionHeader::read(&mut self.backend)?;
            final_checksum += checksum;
            self.sections.insert(self.next_handle, SectionEntry {
                header,
                data: None,
                modified: false,
                index: i,
                entry1: SectionEntry1 {
                    flags: header.flags,
                    threshold: DEFAULT_COMPRESSION_THRESHOLD
                }
            });
            self.next_handle += 1;
        }
        if final_checksum != self.main_header.chksum {
            return Err(ReadError::Checksum(final_checksum, self.main_header.chksum));
        }
        Ok(())
    }

    /// Creates a new BPX decoder.
    ///
    /// # Arguments
    ///
    /// * `file`: An [IoBackend](self::IoBackend) to use for reading the data.
    ///
    /// returns: Result<Decoder<TBackend>, Error>
    ///
    /// # Errors
    ///
    /// A [ReadError](crate::error::ReadError) is returned if some headers
    /// could not be read or if the header data is corrupted.
    pub fn open(mut backend: T) -> Result<Container<T>, ReadError>
    {
        let (checksum, header) = MainHeader::read(&mut backend)?;
        let mut container = Container {
            backend,
            main_header: header,
            sections: BTreeMap::new(),
            next_handle: 0,
            modified: false
        };
        container.read_section_header_table(checksum)?;
        Ok(container)
    }
}

impl<T: io::Write + io::Seek> Container<T>
{
    pub fn create<H: Into<MainHeader>>(backend: T, header: H) -> Container<T>
    {
        Container {
            backend,
            modified: true,
            main_header: header.into(),
            next_handle: 0,
            sections: BTreeMap::new()
        }
    }

    fn write_sections(&mut self, file_start_offset: usize) -> Result<(u32, usize), WriteError>
    {
        let mut ptr: u64 = file_start_offset as _;
        let mut all_sections_size: usize = 0;
        let mut chksum_sht: u32 = 0;

        for (idx, (_handle, section)) in self.sections.iter_mut().enumerate() {
            //At this point the handle must be valid otherwise sections_in_order is broken
            let data = section.data.as_mut().ok_or_else(|| WriteError::SectionNotLoaded)?;
            if data.size() > u32::MAX as usize {
                return Err(WriteError::Capacity(data.size()));
            }
            let last_section_ptr = data.stream_position()?;
            data.seek(io::SeekFrom::Start(0))?;
            let flags = section.entry1.get_flags(data.size() as u32);
            let (csize, chksum) = write_section(flags, data.as_mut(), &mut self.backend)?;
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
                self.backend.seek(SeekFrom::Start(header_start_offset as _))?;
                section.header.write(&mut self.backend)?;
                //Reset file pointer back to the end of the last written section
                self.backend.seek(SeekFrom::Start(ptr))?;
            }
            chksum_sht += section.header.get_checksum();
            all_sections_size += csize;
        }
        Ok((chksum_sht, all_sections_size))
    }

    fn internal_save(&mut self) -> Result<(), WriteError>
    {
        let file_start_offset =
            SIZE_MAIN_HEADER + (SIZE_SECTION_HEADER * self.main_header.section_num as usize);
        //Seek to the start of the actual file content
        self.backend.seek(SeekFrom::Start(file_start_offset as _))?;
        //Write all section data and section headers
        let (chksum_sht, all_sections_size) = self.write_sections(file_start_offset)?;
        self.main_header.file_size = all_sections_size as u64 + file_start_offset as u64;
        self.main_header.chksum = 0;
        self.main_header.chksum = chksum_sht + self.main_header.get_checksum();
        //Relocate to the start of the file and write the BPX main header
        self.backend.seek(SeekFrom::Start(0))?;
        self.main_header.write(&mut self.backend)?;
        self.modified = false;
        Ok(())
    }

    fn write_last_section(&mut self, last_handle: u32) -> Result<(bool, i64), WriteError>
    {
        let entry = self.sections.get_mut(&last_handle).unwrap();
        self.backend.seek(SeekFrom::Start(entry.header.pointer))?;
        let data = entry.data.as_mut().ok_or_else(|| WriteError::SectionNotLoaded)?;
        let last_section_ptr = data.stream_position()?;
        let flags = entry.entry1.get_flags(data.size() as u32);
        let (csize, chksum) = write_section(flags, data.as_mut(), &mut self.backend)?;
        data.seek(io::SeekFrom::Start(last_section_ptr))?;
        let old = entry.header;
        entry.header.csize = csize as u32;
        entry.header.size = data.size() as u32;
        entry.header.chksum = chksum;
        entry.header.flags = flags;
        let diff = entry.header.csize as i64 - old.csize as i64;
        Ok((old == entry.header, diff))
    }

    fn internal_save_last(&mut self) -> Result<(), WriteError>
    {
        // This function saves only the last section.
        let (update_sht, diff) = self.write_last_section(self.next_handle - 1)?;
        if update_sht {
            let offset_section_header = SIZE_MAIN_HEADER
                + (SIZE_SECTION_HEADER * (self.main_header.section_num - 1) as usize);
            self.backend
                .seek(SeekFrom::Start(offset_section_header as _))?;
            let entry = &self.sections[&(self.next_handle - 1)];
            entry.header.write(&mut self.backend)?;
        }
        if diff != 0 {
            self.backend.seek(SeekFrom::Start(0))?;
            self.main_header.file_size = self.main_header.file_size.wrapping_add(diff as u64);
            self.main_header.write(&mut self.backend)?;
        }
        Ok(())
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
    pub fn save(&mut self) -> Result<(), WriteError>
    {
        let mut filter = self
            .sections
            .iter()
            .filter(|(_, entry)| entry.modified);
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
        Ok(())
    }
}
