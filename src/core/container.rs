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

use std::{cell::RefCell, collections::BTreeMap, io};

use crate::core::encoder::{Encoder, SaveMode};
use crate::core::{decoder::read_section_header_table, header::{MainHeader, Struct}, section::SectionTable, Result, SectionData, AutoSectionData};

use super::compression::{WeakChecksum, Checksum};
use super::header::GetChecksum;
use super::options::{CreateOptions, OpenOptions};
use super::error::Error;

/// The default maximum size of uncompressed sections.
///
/// *Used as default compression threshold when a section is marked as compressible.*
pub const DEFAULT_COMPRESSION_THRESHOLD: u32 = 65536;

/// The default maximum size of a section stored in memory (RAM) in bytes.
pub const DEFAULT_MEMORY_THRESHOLD: u32 = 100000000;

/// The main BPX container implementation.
pub struct Container<T> {
    table: SectionTable<T>,
    main_header: MainHeader,
    main_header_modified: bool,
    revert_on_save_failure: bool
}

impl<T> Container<T> {
    /// Returns a mutable reference to the main header.
    ///
    /// **NOTE: This function marks the BPX Main Header as changed.**
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::Container;
    /// use bpx::util::new_byte_buf;
    ///
    /// let mut file = Container::create(new_byte_buf(0));
    /// file.main_header_mut().ty = 1;
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
    /// use bpx::core::Container;
    /// use bpx::util::new_byte_buf;
    ///
    /// let file = Container::create(new_byte_buf(0));
    /// let header = file.main_header();
    /// //Default BPX variant/type is 0.
    /// assert_eq!(header.ty, 0);
    /// ```
    pub fn main_header(&self) -> &MainHeader {
        &self.main_header
    }

    /// Returns a read-only reference to the BPX main header.
    #[deprecated(note = "use `main_header`")]
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
    /// use bpx::core::Container;
    /// use bpx::util::new_byte_buf;
    ///
    /// let mut file = Container::create(new_byte_buf(0));
    /// file.save().unwrap();
    /// let mut buf = file.into_inner();
    /// buf.set_position(0);
    /// let file = Container::open(buf).unwrap();
    /// //Default BPX variant/type is 0.
    /// assert_eq!(file.main_header().ty, 0);
    /// ```
    pub fn open(options: impl Into<OpenOptions<T>>) -> Result<Container<T>> {
        let mut options = options.into();
        let mut checksum = WeakChecksum::default();
        let header = match MainHeader::read(&mut options.backend) {
            Ok(v) => v,
            Err(e) => {
                match e.error() {
                    Error::BadSignature(_) => match options.skip_signature_check {
                        true => e.unwrap_value(),
                        false => return e.into()
                    },
                    Error::BadVersion(_) => match options.skip_version_check {
                        true => e.unwrap_value(),
                        false => return e.into()
                    },
                    _ => return e.into()
                }
            }
        };
        header.get_checksum(&mut checksum);
        let (next_handle, sections) = read_section_header_table(&mut options.backend, &header, &mut checksum)?;
        let chksum = checksum.finish();
        if !options.skip_checksum && chksum != header.chksum {
            return Err(Error::Checksum {
                actual: chksum,
                expected: header.chksum,
            });
        }
        Ok(Container {
            table: SectionTable {
                backend: RefCell::new(options.backend),
                next_handle,
                modified: false,
                sections,
                count: header.section_num,
                skip_checksum: options.skip_checksum,
                memory_threshold: options.memory_threshold
            },
            main_header: header,
            main_header_modified: false,
            revert_on_save_failure: options.revert_on_save_fail
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
    /// use bpx::core::Container;
    /// use bpx::util::new_byte_buf;
    ///
    /// let mut file = Container::create(new_byte_buf(0));
    /// assert_eq!(file.main_header().section_num, 0);
    /// //Default BPX variant/type is 0.
    /// assert_eq!(file.main_header().ty, 0);
    /// ```
    pub fn create(options: impl Into<CreateOptions<T>>) -> Container<T> {
        let options = options.into();
        Container {
            table: SectionTable {
                next_handle: 0,
                count: 0,
                modified: true,
                backend: RefCell::new(options.backend),
                sections: BTreeMap::new(),
                skip_checksum: false,
                memory_threshold: options.memory_threshold
            },
            main_header: options.header.into(),
            main_header_modified: true,
            revert_on_save_failure: options.revert_on_save_fail
        }
    }

    fn get_save_mode(&self) -> SaveMode {
        if self.table.modified {
            return SaveMode::Regenerate;
        }
        let count = self
            .table
            .sections
            .values()
            .filter(|entry| entry.modified.get())
            .count();
        if count == 0 {
            // No sections have changed and the table didn't change; might have nothing to do.
            return SaveMode::MainHeaderOnly
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
            if count > 1 {
                // If n sections changed but did not expand -> only write these n sections and patch section header table.
                return SaveMode::PatchMultipleSections;
            } else {
                // If 1 section changed but did not expand -> only write this section and patch section header table.
                let section = self
                    .table
                    .sections
                    .iter()
                    .find(|(_, entry)| entry.modified.get())
                    .map(|(handle, _)| *handle)
                    .unwrap();

                return SaveMode::PatchSingleSection(section);
            }
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
                if count > 1 {
                    // If n sections changed but did not expand and the last section has expanded
                    // -> write only these n sections, write the last section and patch section
                    // header table.
                    return SaveMode::PatchMultipleSections;
                } else {
                    //If last section expanded -> only write last section and update section header table.
                    return SaveMode::PatchSingleSection(expanded_section);
                }
            }
        }
        SaveMode::Regenerate
    }

    fn save_with_mode_direct(&mut self, mode: SaveMode) -> Result<()> {
        Encoder {
            mode,
            main_header: &mut self.main_header,
            sections: &mut self.table.sections,
            main_header_modified: &mut self.main_header_modified,
            table_modified: &mut self.table.modified,
            table_count: self.table.count
        }.run(self.table.backend.get_mut())
    }

    fn save_with_mode_indirect(&mut self, mode: SaveMode) -> Result<()> {
        use std::io::Seek;
        let mut temp = AutoSectionData::new(self.table.memory_threshold);
        let res = Encoder {
            mode,
            main_header: &mut self.main_header,
            sections: &mut self.table.sections,
            main_header_modified: &mut self.main_header_modified,
            table_modified: &mut self.table.modified,
            table_count: self.table.count
        }.run(&mut temp);
        if res.is_ok() {
            let backend = self.table.backend.get_mut();
            backend.seek(io::SeekFrom::Start(0))?;
            temp.seek(io::SeekFrom::Start(0))?;
            std::io::copy(&mut temp, backend)?;
        }
        res
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
    /// use bpx::core::Container;
    /// use bpx::util::new_byte_buf;
    ///
    /// let mut file = Container::create(new_byte_buf(0));
    /// file.save().unwrap();
    /// let buf = file.into_inner();
    /// assert!(!buf.into_inner().is_empty());
    /// ```
    pub fn save(&mut self) -> Result<()> {
        match self.revert_on_save_failure {
            true => self.save_with_mode_indirect(self.get_save_mode()),
            false => self.save_with_mode_direct(self.get_save_mode())
        }
    }
}

impl<T: io::Read + io::Write + io::Seek> Container<T> {
    /// Loads sections on demand and saves all changes to the underlying IO backend.
    /// This allows saving a BPX container event when all sections have not already been loaded.
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
    /// use bpx::core::Container;
    /// use bpx::util::new_byte_buf;
    ///
    /// let mut file = Container::create(new_byte_buf(0));
    /// file.load_and_save().unwrap();
    /// let buf = file.into_inner();
    /// assert!(!buf.into_inner().is_empty());
    /// ```
    pub fn load_and_save(&mut self) -> Result<()> {
        let mode = self.get_save_mode();
        match mode {
            SaveMode::Regenerate => {
                for handle in self.sections() {
                    self.sections().load(handle)?;
                }
            },
            _ => ()
        }
        match self.revert_on_save_failure {
            true => self.save_with_mode_indirect(mode),
            false => self.save_with_mode_direct(mode)
        }
    }
}
