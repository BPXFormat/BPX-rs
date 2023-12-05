// Copyright (c) 2023, BlockProject 3D
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
    cell::{Cell, RefCell, RefMut},
    collections::{btree_map::Keys, BTreeMap, Bound},
    io::{Read, Seek},
};
use std::ops::Index;

use crate::core::{
    data::AutoSectionData,
    decoder::load_section1,
    error::{Error, OpenError},
    header::{
        SectionHeader, FLAG_CHECK_CRC32, FLAG_CHECK_WEAK, FLAG_COMPRESS_XZ, FLAG_COMPRESS_ZLIB,
    },
    Result,
};

/// Represents a pointer to a section.
///
/// *Allows indirect access to a given section instead of sharing mutable references in user code.*
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Handle(u32);

impl Handle {
    /// Constructs a Handle from a raw u32.
    ///
    /// # Arguments
    ///
    /// * `raw`: the raw key.
    ///
    /// returns: Handle
    ///
    /// # Safety
    ///
    /// You must ensure the raw key is a valid key. Failure to do so could panic bpx::core::Container.
    pub unsafe fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Extracts the raw key from this Handle.
    pub fn into_raw(self) -> u32 {
        self.0
    }
}

pub struct SectionEntry1 {
    pub threshold: u32,
    pub flags: u8,
}

impl SectionEntry1 {
    pub fn get_flags(&self, size: u32) -> u8 {
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

/// Information about a section.
pub struct SectionInfo {
    index: u32,
    header: SectionHeader
}

impl SectionInfo {
    /// Creates a new [SectionInfo].
    ///
    /// # Arguments
    ///
    /// * `index`: section index.
    /// * `header`: section header.
    pub fn new(index: u32, header: SectionHeader) -> SectionInfo {
        SectionInfo {
            index,
            header
        }
    }

    /// Returns a reference to the section header.
    pub fn header(&self) -> &SectionHeader {
        &self.header
    }

    /// Returns a mutable reference to the section header.
    pub fn header_mut(&mut self) -> &mut SectionHeader {
        &mut self.header
    }

    /// Returns the index of the section.
    pub fn index(&self) -> u32 {
        self.index
    }
}

pub struct SectionEntry {
    pub(crate) entry1: SectionEntry1,
    pub(crate) info: SectionInfo,
    pub(crate) data: RefCell<Option<AutoSectionData>>,
    pub(crate) modified: Cell<bool>,
}

/// An iterator over section handles.
pub struct Iter<'a> {
    iter: Keys<'a, u32, SectionEntry>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Handle;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|v| Handle(*v))
    }
}

/// Represents the table of all sections in a BPX [Container](crate::core::Container).
pub struct SectionTable<T> {
    pub(crate) backend: RefCell<T>,
    pub(crate) sections: BTreeMap<u32, SectionEntry>,
    pub(crate) count: u32,
    pub(crate) modified: bool,
    pub(crate) next_handle: u32,
    pub(crate) skip_checksum: bool,
    pub(crate) memory_threshold: u32,
}

impl<T: Read + Seek> SectionTable<T> {
    /// Opens a section for reading and/or writing. Loads the section if needed.
    ///
    /// # Panics
    ///
    /// Panics if the given section handle is invalid.
    ///
    /// # Errors
    ///
    /// Returns an [Error](Error) if the section is corrupted,
    /// truncated, if some data couldn't be read or if the section is already in use.
    pub fn load(&self, handle: Handle) -> Result<RefMut<AutoSectionData>> {
        let section = &self.sections[&handle.0];
        let mut data = section
            .data
            .try_borrow_mut()
            .map_err(|_| Error::Open(OpenError::SectionInUse))?;
        if data.is_none() {
            let mut backend = self.backend.borrow_mut();
            let loaded = match self.skip_checksum {
                false => load_section1(&mut *backend, &section.info.header, self.memory_threshold)?,
                true => {
                    let mut header = section.info.header;
                    //If checksum is to be ignored, clear the checksum flags before loading the section.
                    header.flags &= !FLAG_CHECK_WEAK;
                    header.flags &= !FLAG_CHECK_CRC32;
                    load_section1(&mut *backend, &section.info.header, self.memory_threshold)?
                },
            };
            *data = Some(loaded);
        }
        section.modified.set(true);
        Ok(RefMut::map(data, |v| unsafe {
            v.as_mut().unwrap_unchecked()
        }))
    }
}

impl<T> SectionTable<T> {
    /// Opens a section for reading and/or writing.
    ///
    /// # Arguments
    ///
    /// * `handle`: a handle to the section.
    ///
    /// returns: `Result<RefMut<AutoSectionData>, OpenError>`
    ///
    /// # Panics
    ///
    /// Panics if the given section handle is invalid.
    ///
    /// # Errors
    ///
    /// Returns an [OpenError](OpenError) if the section is already in use
    /// or is not loaded. To ensure a section is loaded, call load.
    pub fn open(&self, handle: Handle) -> std::result::Result<RefMut<AutoSectionData>, OpenError> {
        let section = &self.sections[&handle.0];
        let data = section
            .data
            .try_borrow_mut()
            .map_err(|_| OpenError::SectionInUse)?;
        if data.is_none() {
            return Err(OpenError::SectionNotLoaded);
        }
        section.modified.set(true);
        Ok(RefMut::map(data, |v| unsafe {
            v.as_mut().unwrap_unchecked()
        }))
    }

    /// Returns true if this section table contains no section.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the number of sections in this table.
    pub fn len(&self) -> u32 {
        self.count
    }

    /// Returns an iterator over all section handles in this table.
    pub fn iter(&self) -> Iter {
        Iter {
            iter: self.sections.keys(),
        }
    }

    /// Creates a new section in the section table.
    ///
    /// # Arguments
    ///
    /// * `header`: the [SectionHeader](SectionHeader) of the new section.
    ///
    /// returns: Handle
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::{SectionOptions};
    /// use bpx::core::{Container, SectionData};
    /// use bpx::util::new_byte_buf;
    ///
    /// let mut file = Container::create(new_byte_buf(0));
    /// assert_eq!(file.sections().len(), 0);
    /// file.sections_mut().create(SectionOptions::new());
    /// assert_eq!(file.sections().len(), 1);
    /// ```
    pub fn create<H: Into<SectionHeader>>(&mut self, header: H) -> Handle {
        self.modified = true;
        self.count += 1;
        let r = self.next_handle;
        let section = AutoSectionData::new(self.memory_threshold as _);
        let h = header.into();
        let entry = SectionEntry {
            info: SectionInfo::new(self.count - 1, h),
            data: RefCell::new(Some(section)),
            modified: Cell::new(false),
            entry1: SectionEntry1 {
                threshold: h.csize,
                flags: h.flags,
            },
        };
        self.sections.insert(r, entry);
        self.next_handle += 1;
        Handle(r)
    }

    /// Removes a section from this section table.
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
    /// use bpx::core::options::{SectionOptions};
    /// use bpx::core::{Container, SectionData};
    /// use bpx::util::new_byte_buf;
    ///
    /// let mut file = Container::create(new_byte_buf(0));
    /// let section = file.sections_mut().create(SectionOptions::new());
    /// file.save().unwrap();
    /// assert_eq!(file.main_header().section_num, 1);
    /// file.sections_mut().remove(section);
    /// file.save().unwrap();
    /// assert_eq!(file.main_header().section_num, 0);
    /// ```
    pub fn remove(&mut self, handle: Handle) {
        self.sections.remove(&handle.0);
        self.count -= 1;
        self.modified = true;
        self.sections
            .range_mut((Bound::Included(handle.0), Bound::Unbounded))
            .for_each(|(_, v)| {
                v.info.index -= 1;
            });
    }

    /// Searches for the first section of a given type.
    /// Returns None if no section could be found.
    ///
    /// # Arguments
    ///
    /// * `ty`: section type byte.
    ///
    /// returns: `Option<Handle>`
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::Container;
    /// use bpx::util::new_byte_buf;
    ///
    /// let file = Container::create(new_byte_buf(0));
    /// assert!(file.sections().find_by_type(0).is_none());
    /// ```
    pub fn find_by_type(&self, ty: u8) -> Option<Handle> {
        for (handle, entry) in &self.sections {
            if entry.info.header.ty == ty {
                return Some(Handle(*handle));
            }
        }
        None
    }

    /// Locates a section by its index in the file.
    /// Returns None if the section does not exist.
    ///
    /// # Arguments
    ///
    /// * `index`: the section index to search for.
    ///
    /// returns: `Option<Handle>`
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::Container;
    /// use bpx::util::new_byte_buf;
    ///
    /// let file = Container::create(new_byte_buf(0));
    /// assert!(file.sections().find_by_index(0).is_none());
    /// ```
    pub fn find_by_index(&self, index: u32) -> Option<Handle> {
        for (idx, handle) in self.sections.keys().enumerate() {
            if idx as u32 == index {
                return Some(Handle(*handle));
            }
        }
        None
    }
}

impl<'a, T> IntoIterator for &'a SectionTable<T> {
    type Item = Handle;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> Index<Handle> for SectionTable<T> {
    type Output = SectionInfo;

    fn index(&self, index: Handle) -> &Self::Output {
        &self.sections[&index.0].info
    }
}
