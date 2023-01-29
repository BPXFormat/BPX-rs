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

use std::io::SeekFrom;
use std::{cell::RefCell, collections::BTreeMap, io};

use crate::core::encoder::recompute_header_checksum;
use crate::core::{
    decoder::read_section_header_table,
    encoder::{internal_save, internal_save_single},
    header::{MainHeader, Struct},
    section::SectionTable,
    Result, SectionData,
};

/// The default maximum size of uncompressed sections.
///
/// *Used as default compression threshold when a section is marked as compressible.*
pub const DEFAULT_COMPRESSION_THRESHOLD: u32 = 65536;

/// The main BPX container implementation.
pub struct Container<T> {
    table: SectionTable<T>,
    main_header: MainHeader,
    main_header_modified: bool,
}

impl<T> Container<T> {
    /// Sets the BPX Main Header.
    ///
    /// # Arguments
    ///
    /// * `main_header`: the new [MainHeader](MainHeader).
    #[deprecated(note="use `main_header_mut`")]
    pub fn set_main_header<H: Into<MainHeader>>(&mut self, main_header: H) {
        self.main_header = main_header.into();
        self.main_header_modified = true;
    }

    /// Returns a mutable reference to the main header.
    ///
    /// **NOTE: This function marks the BPX Main Header as changed.**
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::MainHeaderBuilder;
    /// use bpx::core::Container;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut file = Container::create(new_byte_buf(0), MainHeaderBuilder::new());
    /// *file.main_header_mut() = MainHeaderBuilder::new().ty(1).into();
    /// assert_eq!(file.main_header().ty, 1);
    /// ```
    pub fn main_header_mut(&mut self) -> &mut MainHeader {
        self.main_header_modified = true;
        &mut self.main_header
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
    /// let header = file.main_header();
    /// //Default BPX variant/type is 0.
    /// assert_eq!(header.ty, 0);
    /// ```
    pub fn main_header(&self) -> &MainHeader {
        &self.main_header
    }

    /// Returns a read-only reference to the BPX main header.
    #[deprecated(note="use `main_header`")]
    pub fn get_main_header(&self) -> &MainHeader {
        &self.main_header
    }

    /// Consumes this BPX container and returns the inner IO backend.
    pub fn into_inner(self) -> T {
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

impl<T: io::Read + io::Seek> Container<T> {
    /// Loads a BPX container from the given `backend`.
    ///
    /// # Arguments
    ///
    /// * `backend`: A [Read](io::Read) + [Seek](io::Seek) backend to use for reading the BPX container.
    ///
    /// returns: `Result<Decoder<TBackend>>`
    ///
    /// # Errors
    ///
    /// An [Error](crate::core::error::Error) is returned if some headers
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
    /// //Default BPX variant/type is 0.
    /// assert_eq!(file.main_header().ty, 0);
    /// ```
    pub fn open(mut backend: T) -> Result<Container<T>> {
        let (checksum, header) = MainHeader::read(&mut backend)?;
        let (next_handle, sections) = read_section_header_table(&mut backend, &header, checksum)?;
        Ok(Container {
            table: SectionTable {
                backend: RefCell::new(backend),
                next_handle,
                modified: false,
                sections,
                count: header.section_num,
            },
            main_header: header,
            main_header_modified: false,
        })
    }
}

impl<T: io::Write + io::Seek> Container<T> {
    /// Creates a new BPX container in the given `backend`.
    ///
    /// # Arguments
    ///
    /// * `backend`: A [Write](io::Write) + [Seek](io::Seek) backend to use for writing the BPX container.
    /// * `header`: The [MainHeader](MainHeader) to initialize the new container.
    ///
    /// returns: `Container<T>`
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::MainHeaderBuilder;
    /// use bpx::core::Container;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut file = Container::create(new_byte_buf(0), MainHeaderBuilder::new());
    /// assert_eq!(file.main_header().section_num, 0);
    /// //Default BPX variant/type is 0.
    /// assert_eq!(file.main_header().ty, 0);
    /// ```
    pub fn create<H: Into<MainHeader>>(backend: T, header: H) -> Container<T> {
        Container {
            table: SectionTable {
                next_handle: 0,
                count: 0,
                modified: true,
                backend: RefCell::new(backend),
                sections: BTreeMap::new(),
            },
            main_header: header.into(),
            main_header_modified: true,
        }
    }

    fn get_modified_sections(&self) -> Vec<u32> {
        // Returns a list of modified sections.
        self.table
            .sections
            .iter()
            .filter(|(_, entry)| entry.modified.get())
            .map(|(handle, _)| *handle)
            .collect()
    }

    fn patch_main_header_if_needed(&mut self, was_written: bool) -> Result<()> {
        if was_written {
            self.main_header_modified = false;
            Ok(())
        } else if self.main_header_modified {
            // If only main header changed -> write only main header.
            self.main_header_modified = false;
            recompute_header_checksum(&mut self.main_header, &self.table.sections);
            let backend = self.table.backend.get_mut();
            backend.seek(SeekFrom::Start(0))?;
            self.main_header.write(backend)?;
            Ok(())
        } else {
            Ok(())
        }
    }

    fn patch_modified_sections(&mut self) -> Result<bool> {
        let mut main_header = false;
        for section in self.get_modified_sections() {
            if internal_save_single(
                self.table.backend.get_mut(),
                &mut self.table.sections,
                &mut self.main_header,
                section,
            )? {
                main_header = true;
            }
        }
        Ok(main_header)
    }

    /// Writes all sections to the underlying IO backend.
    ///
    /// **This function prints some information to standard output as a way
    /// to debug data compression issues unless the `debug-log` feature
    /// is disabled.**
    ///
    /// # Errors
    ///
    /// An [Error](crate::core::error::Error) is returned if some data could
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
    pub fn save(&mut self) -> Result<()> {
        if self.table.modified {
            // If table changed -> write all file.
            self.main_header_modified = false;
            self.table.modified = false;
            self.main_header.section_num = self.table.count;
            return internal_save(
                self.table.backend.get_mut(),
                &mut self.table.sections,
                &mut self.main_header,
            );
        }
        let count = self
            .table
            .sections
            .values()
            .filter(|entry| entry.modified.get())
            .count();
        if count == 0 {
            // No sections have changed and the table didn't change; might have nothing to do.
            return self.patch_main_header_if_needed(false);
        }
        let expanded_sections = self
            .table
            .sections
            .values()
            .filter(|entry| entry.modified.get())
            .filter(|entry| {
                entry.data.borrow().as_ref().unwrap().size() != entry.header.size as usize
            })
            .count();
        if expanded_sections == 0 {
            let flag = if count > 1 {
                // If n sections changed but did not expand -> only write these n sections and patch section header table.
                self.patch_modified_sections()
            } else {
                // If 1 section changed but did not expand -> only write this section and patch section header table.
                let section = self
                    .table
                    .sections
                    .iter()
                    .find(|(_, entry)| entry.modified.get())
                    .map(|(handle, _)| *handle)
                    .unwrap();
                internal_save_single(
                    self.table.backend.get_mut(),
                    &mut self.table.sections,
                    &mut self.main_header,
                    section,
                )
            };
            return self.patch_main_header_if_needed(flag?);
        }
        if expanded_sections == 1 {
            let expanded_section = self
                .table
                .sections
                .iter()
                .filter(|(_, entry)| entry.modified.get())
                .find(|(_, entry)| {
                    entry.data.borrow().as_ref().unwrap().size() != entry.header.size as usize
                })
                .map(|(handle, _)| *handle)
                .unwrap();
            if expanded_section == self.table.next_handle - 1 {
                let flag = if count > 1 {
                    // If n sections changed but did not expand and the last section has expanded
                    // -> write only these n sections, write the last section and patch section
                    // header table.
                    self.patch_modified_sections()
                } else {
                    //If last section expanded -> only write last section and update section header table.
                    internal_save_single(
                        self.table.backend.get_mut(),
                        &mut self.table.sections,
                        &mut self.main_header,
                        expanded_section,
                    )
                };
                return self.patch_main_header_if_needed(flag?);
            }
        }
        self.main_header_modified = false;
        self.table.modified = false;
        self.main_header.section_num = self.table.count;
        internal_save(
            self.table.backend.get_mut(),
            &mut self.table.sections,
            &mut self.main_header,
        )
    }
}
