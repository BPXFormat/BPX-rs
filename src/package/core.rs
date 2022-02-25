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

use std::io::{Read, Seek, SeekFrom, Write};
use once_cell::unsync::OnceCell;

use crate::{
    core::{
        builder::{Checksum, CompressionMethod, MainHeaderBuilder, SectionHeaderBuilder},
        header::{Struct, SECTION_TYPE_SD, SECTION_TYPE_STRING},
        Container
    },
    package::{
        decoder::{get_arch_platform_from_code, read_object_table},
        encoder::get_type_ext,
        error::{ReadError, Section, WriteError},
        Settings,
        SECTION_TYPE_OBJECT_TABLE,
        SUPPORTED_VERSION
    },
    strings::StringSection
};
use crate::core::Handle;
use crate::package::table::{ObjectTable, ObjectTableMut, ObjectTableRef};
use crate::table::NamedItemTable;

/// A BPXP (Package).
///
/// # Examples
///
/// ```
/// use std::io::{Seek, SeekFrom};
/// use bpx::utils::new_byte_buf;
/// use bpx::package::{Builder, Package};
///
/// let mut bpxp = Package::create(new_byte_buf(128), Builder::new()).unwrap();
/// {
///     let mut objects = bpxp.objects_mut().unwrap();
///     objects.create("TestObject", "This is a test 你好".as_bytes()).unwrap();
/// }
/// bpxp.save().unwrap();
/// //Reset our bytebuf pointer to start
/// let mut bytebuf = bpxp.into_inner().into_inner();
/// bytebuf.seek(SeekFrom::Start(0)).unwrap();
/// //Attempt decoding our in-memory BPXP
/// let mut bpxp = Package::open(bytebuf).unwrap();
/// let objects = bpxp.objects().unwrap();
/// assert_eq!(objects.len(), 1);
/// let last = objects.iter().last().unwrap();
/// assert_eq!(objects.load_name(last).unwrap(), "TestObject");
/// {
///     let mut data = Vec::new();
///     objects.load(last, &mut data).unwrap();
///     let s = std::str::from_utf8(&data).unwrap();
///     assert_eq!(s, "This is a test 你好")
/// }
/// {
///     let wanted = objects.find("TestObject").unwrap().unwrap();
///     let mut data = Vec::new();
///     objects.load(wanted, &mut data).unwrap();
///     let s = std::str::from_utf8(&data).unwrap();
///     assert_eq!(s, "This is a test 你好")
/// }
/// ```
pub struct Package<T>
{
    settings: Settings,
    container: Container<T>,
    object_table: Handle,
    table: OnceCell<ObjectTable>,
    metadata: OnceCell<Option<crate::sd::Object>>
}

impl<T> Package<T>
{
    /// Gets the settings of this package.
    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }

    /// Consumes this Package and returns the inner BPX container.
    pub fn into_inner(self) -> Container<T>
    {
        self.container
    }
}

impl<T: Write + Seek> Package<T>
{
    /// Creates a new BPX type P.
    ///
    /// # Arguments
    ///
    /// * `backend`: A [Write](std::io::Write) + [Seek](std::io::Seek) to use as backend.
    /// * `settings`: The package creation settings.
    ///
    /// returns: Result<Package<T>, WriteError>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::package::Builder;
    /// use bpx::package::Package;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut bpxp = Package::create(new_byte_buf(0), Builder::new()).unwrap();
    /// bpxp.save();
    /// assert!(!bpxp.into_inner().into_inner().into_inner().is_empty());
    /// ```
    pub fn create<S: Into<Settings>>(backend: T, settings: S) -> Result<Package<T>, WriteError>
    {
        let settings = settings.into();
        let mut container = Container::create(
            backend,
            MainHeaderBuilder::new()
                .ty(b'P')
                .type_ext(get_type_ext(&settings))
                .version(SUPPORTED_VERSION)
        );
        let object_table = container.sections_mut().create(
            SectionHeaderBuilder::new()
                .checksum(Checksum::Weak)
                .compression(CompressionMethod::Zlib)
                .ty(SECTION_TYPE_OBJECT_TABLE)
        );
        let string_section = container.sections_mut().create(
            SectionHeaderBuilder::new()
                .checksum(Checksum::Weak)
                .compression(CompressionMethod::Zlib)
                .ty(SECTION_TYPE_STRING)
        );
        let strings = StringSection::new(string_section);
        if let Some(metadata) = &settings.metadata {
            let metadata_section = container.sections_mut().create(
                SectionHeaderBuilder::new()
                    .checksum(Checksum::Weak)
                    .compression(CompressionMethod::Zlib)
                    .ty(SECTION_TYPE_SD)
            );
            let mut section = container.sections().open(metadata_section)?;
            metadata.write(&mut *section)?;
        }
        Ok(Package {
            metadata: OnceCell::from(settings.metadata.clone()),
            settings,
            container,
            object_table,
            table: OnceCell::from(ObjectTable::new(NamedItemTable::empty(), strings))
        })
    }

    /// Saves this package.
    ///
    /// # Errors
    ///
    /// Returns a [WriteError](crate::package::error::WriteError) if some parts of this package
    /// couldn't be saved.
    pub fn save(&mut self) -> Result<(), WriteError>
    {
        {
            let mut section = self.container.sections().open(self.object_table)?;
            section.seek(SeekFrom::Start(0))?;
            if let Some(val) = self.table.get() {
                for v in val {
                    v.write(&mut *section)?;
                }
            }
        }
        self.container.save()?;
        Ok(())
    }
}

impl<T: Read + Seek> Package<T>
{
    /// Opens a BPX type P.
    ///
    /// # Arguments
    ///
    /// * `backend`: A [Read](std::io::Read) + [Seek](std::io::Seek) to use as backend.
    ///
    /// returns: Result<PackageDecoder<TBackend>, Error>
    ///
    /// # Errors
    ///
    /// A [ReadError](crate::package::error::ReadError) is returned if some
    /// sections/headers could not be loaded.
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::package::Builder;
    /// use bpx::package::Package;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut bpxp = Package::create(new_byte_buf(0), Builder::new()).unwrap();
    /// bpxp.save();
    /// let mut buf = bpxp.into_inner().into_inner();
    /// buf.set_position(0);
    /// let mut bpxp = Package::open(buf).unwrap();
    /// let objects = bpxp.objects().unwrap();
    /// assert_eq!(objects.len(), 0);
    /// ```
    pub fn open(backend: T) -> Result<Package<T>, ReadError>
    {
        let container = Container::open(backend)?;
        if container.get_main_header().ty != b'P' {
            return Err(ReadError::BadType(container.get_main_header().ty));
        }
        if container.get_main_header().version != SUPPORTED_VERSION {
            return Err(ReadError::BadVersion(container.get_main_header().version));
        }
        let (a, p) = get_arch_platform_from_code(
            container.get_main_header().type_ext[0],
            container.get_main_header().type_ext[1]
        )?;
        let object_table = match container.sections().find_by_type(SECTION_TYPE_OBJECT_TABLE) {
            Some(v) => v,
            None => return Err(ReadError::MissingSection(Section::ObjectTable))
        };
        Ok(Self {
            settings: Settings {
                metadata: None,
                architecture: a,
                platform: p,
                type_code: [
                    container.get_main_header().type_ext[2],
                    container.get_main_header().type_ext[3]
                ]
            },
            object_table,
            container,
            table: OnceCell::new(),
            metadata: OnceCell::new()
        })
    }

    fn load_object_table(&self) -> Result<ObjectTable, ReadError> {
        let handle = self.container.sections().find_by_type(SECTION_TYPE_STRING).ok_or(ReadError::MissingSection(Section::Strings))?;
        let strings = StringSection::new(handle);
        let table = read_object_table(&self.container, self.object_table)?;
        Ok(ObjectTable::new(table, strings))
    }

    /// Returns a guard for immutable access to the object table.
    ///
    /// This will load the object table if it's not already loaded.
    ///
    /// # Errors
    ///
    /// A [ReadError](crate::shader::error::ReadError) is returned if the object table could not be
    /// loaded.
    pub fn objects(&self) -> Result<ObjectTableRef<T>, ReadError> {
        let table = self.table.get_or_try_init(|| self.load_object_table())?;
        Ok(ObjectTableRef {
            table,
            container: &self.container
        })
    }

    /// Returns a guard for mutable access to the object table.
    ///
    /// This will load the object table if it's not already loaded.
    ///
    /// # Errors
    ///
    /// A [ReadError](crate::shader::error::ReadError) is returned if the object table could not be
    /// loaded.
    pub fn objects_mut(&mut self) -> Result<ObjectTableMut<T>, ReadError> {
        if self.table.get_mut().is_none() {
            //SAFETY: This is safe because it runs only if the cell is none.
            unsafe { self.table.set(self.load_object_table()?).unwrap_unchecked(); }
        }
        Ok(ObjectTableMut {
            table: unsafe { self.table.get_mut().unwrap_unchecked() },
            container: &mut self.container
        })
    }

    /// Reads the metadata section of this BPXP if any.
    /// Returns None if there is no metadata in this BPXP.
    ///
    /// # Errors
    ///
    /// A [ReadError](crate::package::error::ReadError) is returned in case of corruption or system error.
    pub fn load_metadata(&self) -> Result<Option<&crate::sd::Object>, ReadError> {
        if self.metadata.get().is_none() {
            let res = match self.container.sections().find_by_type(SECTION_TYPE_SD) {
                Some(v) => {
                    let mut section = self.container.sections().load(v)?;
                    let obj = crate::sd::Object::read(&mut *section)?;
                    self.metadata.set(Some(obj))
                },
                None => self.metadata.set(None)
            };
            //SAFETY: This is safe because we're only running this if the cell is none.
            unsafe { res.unwrap_unchecked(); }
        }
        //SAFETY: There's a check right before this line which inserts the value if it doesn't
        // exist.
        unsafe { Ok(self.metadata.get().unwrap_unchecked().as_ref()) }
    }
}
