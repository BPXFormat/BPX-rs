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
    collections::{BTreeMap},
    io
};
use std::cell::RefCell;

use crate::{
    core::{
        decoder::read_section_header_table,
        encoder::{internal_save, internal_save_last},
        error::{ReadError, WriteError},
        header::{MainHeader, Struct},
        section::SectionTable
    }
};

/// The default maximum size of uncompressed sections.
///
/// *Used as default compression threshold when a section is marked as compressible.*
pub const DEFAULT_COMPRESSION_THRESHOLD: u32 = 65536;

/// The main BPX container implementation.
pub struct Container<T>
{
    table: SectionTable<T>,
    main_header: MainHeader
}

impl<T> Container<T>
{
    /// Sets the BPX Main Header.
    ///
    /// # Arguments
    ///
    /// * `main_header`: the new [MainHeader](crate::core::header::MainHeader).
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::MainHeaderBuilder;
    /// use bpx::core::Container;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut file = Container::create(new_byte_buf(0), MainHeaderBuilder::new());
    /// file.set_main_header(MainHeaderBuilder::new().ty(1));
    /// assert_eq!(file.get_main_header().ty, 1);
    /// ```
    pub fn set_main_header<H: Into<MainHeader>>(&mut self, main_header: H)
    {
        self.main_header = main_header.into();
        self.table.modified = true;
    }

    /// Returns a read-only reference to the BPX main header.
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::MainHeaderBuilder;
    /// use bpx::core::Container;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let file = Container::create(new_byte_buf(0), MainHeaderBuilder::new());
    /// let header = file.get_main_header();
    /// //Default BPX variant/type is 'P'
    /// assert_eq!(header.ty, 'P' as u8);
    /// ```
    pub fn get_main_header(&self) -> &MainHeader
    {
        &self.main_header
    }

    /// Consumes this BPX container and returns the inner IO backend.
    pub fn into_inner(self) -> T
    {
        self.table.backend.into_inner()
    }

    /// Gets immutable access to the section table.
    pub fn sections(&self) -> &SectionTable<T> {
        &self.table
    }

    /// Gets mutable access to the section table.
    pub fn sections_mut(&mut self) -> &mut SectionTable<T> {
        &mut self.table
    }
}

impl<T: io::Read + io::Seek> Container<T>
{
    /// Loads a BPX container from the given `backend`.
    ///
    /// # Arguments
    ///
    /// * `backend`: A [Read](std::io::Read) + [Seek](std::io::Seek) backend to use for reading the BPX container.
    ///
    /// returns: Result<Decoder<TBackend>, Error>
    ///
    /// # Errors
    ///
    /// A [ReadError](crate::core::error::ReadError) is returned if some headers
    /// could not be read or if the header data is corrupted.
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::MainHeaderBuilder;
    /// use bpx::core::Container;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut file = Container::create(new_byte_buf(0), MainHeaderBuilder::new());
    /// file.save().unwrap();
    /// let mut buf = file.into_inner();
    /// buf.set_position(0);
    /// let file = Container::open(buf).unwrap();
    /// //Default BPX variant/type is 'P'
    /// assert_eq!(file.get_main_header().ty, 'P' as u8);
    /// ```
    pub fn open(mut backend: T) -> Result<Container<T>, ReadError>
    {
        let (checksum, header) = MainHeader::read(&mut backend)?;
        let (next_handle, sections) = read_section_header_table(&mut backend, &header, checksum)?;
        Ok(Container {
            table: SectionTable {
                backend: RefCell::new(backend),
                next_handle,
                modified: false,
                sections,
                count: header.section_num
            },
            main_header: header
        })
    }
}

impl<T: io::Write + io::Seek> Container<T>
{
    /// Creates a new BPX container in the given `backend`.
    ///
    /// # Arguments
    ///
    /// * `backend`: A [Write](std::io::Write) + [Seek](std::io::Seek) backend to use for writing the BPX container.
    /// * `header`: The [MainHeader](crate::core::header::MainHeader) to initialize the new container.
    ///
    /// returns: Container<T>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::MainHeaderBuilder;
    /// use bpx::core::Container;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut file = Container::create(new_byte_buf(0), MainHeaderBuilder::new());
    /// assert_eq!(file.get_main_header().section_num, 0);
    /// //Default BPX variant/type is 'P'
    /// assert_eq!(file.get_main_header().ty, 'P' as u8);
    /// ```
    pub fn create<H: Into<MainHeader>>(backend: T, header: H) -> Container<T>
    {
        Container {
            table: SectionTable {
                next_handle: 0,
                count: 0,
                modified: true,
                backend: RefCell::new(backend),
                sections: BTreeMap::new()
            },
            main_header: header.into()
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
    /// A [WriteError](crate::core::error::WriteError) is returned if some data could
    /// not be written.
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::MainHeaderBuilder;
    /// use bpx::core::Container;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut file = Container::create(new_byte_buf(0), MainHeaderBuilder::new());
    /// file.save().unwrap();
    /// let buf = file.into_inner();
    /// assert!(!buf.into_inner().is_empty());
    /// ```
    pub fn save(&mut self) -> Result<(), WriteError>
    {
        let mut filter = self.table.sections.iter().filter(|(_, entry)| entry.modified.get());
        let count = filter.by_ref().count();
        if self.table.modified || count > 1 {
            self.table.modified = false;
            self.main_header.section_num = self.table.count;
            internal_save(self.table.backend.get_mut(), &mut self.table.sections, &mut self.main_header)
        } else if !self.table.modified && count == 1 {
            let (handle, _) = filter.last().unwrap();
            if *handle == self.table.next_handle - 1 {
                //Save only the last section (no need to re-write every other section
                internal_save_last(
                    self.table.backend.get_mut(),
                    &mut self.table.sections,
                    &mut self.main_header,
                    self.table.next_handle - 1
                )
            } else {
                //Unfortunately the modified section is not the last one so we can't safely
                //expand/reduce the file size without corrupting other sections
                self.table.modified = false;
                self.main_header.section_num = self.table.count;
                internal_save(self.table.backend.get_mut(), &mut self.table.sections, &mut self.main_header)
            }
        } else {
            Ok(())
        }
    }
}
