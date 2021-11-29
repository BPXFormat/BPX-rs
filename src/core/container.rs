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

use std::{
    collections::{BTreeMap, Bound},
    io
};

use crate::{
    core::{
        data::AutoSectionData,
        decoder::read_section_header_table,
        encoder::{internal_save, internal_save_last},
        error::{ReadError, WriteError},
        header::{MainHeader, SectionHeader, Struct},
        section::{new_section, new_section_mut, SectionEntry, SectionEntry1},
        Section,
        SectionMut
    },
    Handle
};

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
            Some(new_section_mut(&mut *ptr, v, Handle(*h)))
        }
    }
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
        self.sections
            .get(&handle.0)
            .map(|v| new_section(v, handle))
            .expect("attempt to use invalid handle")
    }

    pub fn get_mut(&mut self, handle: Handle) -> SectionMut<T>
    {
        self.sections
            .get_mut(&handle.0)
            .map(|v| new_section_mut(&mut self.backend, v, handle))
            .expect("attempt to use invalid handle")
    }

    /// Creates a new section in the BPX
    ///
    /// # Arguments
    ///
    /// * `header`: the [SectionHeader](crate::header::SectionHeader) of the new section.
    ///
    /// returns: Result<Handle, Error>
    pub fn create_section<H: Into<SectionHeader>>(&mut self, header: H) -> Handle
    {
        self.modified = true;
        self.main_header.section_num += 1;
        let r = self.next_handle;
        let section = AutoSectionData::new();
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
        Handle(r)
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
        self.sections
            .range_mut((Bound::Included(handle.0), Bound::Unbounded))
            .for_each(|(_, v)| {
                v.index -= 1;
            });
    }

    pub fn iter(&self) -> impl Iterator<Item = Section>
    {
        self.sections
            .iter()
            .map(|(h, v)| new_section(v, Handle(*h)))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = SectionMut<T>>
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
        let (next_handle, sections) = read_section_header_table(&mut backend, &header, checksum)?;
        Ok(Container {
            backend,
            main_header: header,
            sections,
            next_handle,
            modified: false
        })
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
        let mut filter = self.sections.iter().filter(|(_, entry)| entry.modified);
        let count = filter.by_ref().count();
        if self.modified || count > 1 {
            self.modified = false;
            internal_save(&mut self.backend, &mut self.sections, &mut self.main_header)
        } else if !self.modified && count == 1 {
            let (handle, _) = filter.last().unwrap();
            if *handle == self.next_handle - 1 {
                //Save only the last section (no need to re-write every other section
                internal_save_last(
                    &mut self.backend,
                    &mut self.sections,
                    &mut self.main_header,
                    self.next_handle - 1
                )
            } else {
                //Unfortunately the modified section is not the last one so we can't safely
                //expand/reduce the file size without corrupting other sections
                self.modified = false;
                internal_save(&mut self.backend, &mut self.sections, &mut self.main_header)
            }
        } else {
            Ok(())
        }
    }
}
