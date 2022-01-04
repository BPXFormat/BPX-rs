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
    io::{Read, Seek, SeekFrom, Write},
    slice::Iter
};

use crate::{
    core::{
        builder::{Checksum, CompressionMethod, MainHeaderBuilder, SectionHeaderBuilder},
        header::{Struct, SECTION_TYPE_SD, SECTION_TYPE_STRING},
        Container,
        SectionData
    },
    package::{
        decoder::{get_arch_platform_from_code, read_object_table, unpack_object},
        encoder::{create_data_section_header, get_type_ext},
        error::{ReadError, Section, WriteError},
        object::ObjectHeader,
        Architecture,
        Platform,
        Settings,
        SECTION_TYPE_OBJECT_TABLE,
        SUPPORTED_VERSION
    },
    strings::{load_string_section, StringSection},
    table::ItemTable,
    utils::{OptionExtension, ReadFill},
    Handle
};

const DATA_WRITE_BUFFER_SIZE: usize = 8192;
const MIN_DATA_REMAINING_SIZE: usize = DATA_WRITE_BUFFER_SIZE;
const MAX_DATA_SECTION_SIZE: usize = 200000000 - MIN_DATA_REMAINING_SIZE; //200MB

/// Represents an object reference.
pub struct Object<'a, T>
{
    container: &'a mut Container<T>,
    strings: &'a mut StringSection,
    header: &'a ObjectHeader
}

impl<'a, T: Read + Seek> Object<'a, T>
{
    /// Unpacks this object to the given `out` io backend.
    ///
    /// # Arguments
    ///
    /// * `out`: A [Write](std::io::Write) to unpack object data to.
    ///
    /// returns: Result<u64, ReadError>
    ///
    /// # Errors
    ///
    /// Returns a [ReadError](crate::package::error::ReadError) if the section couldn't be loaded
    /// or an IO error has occured.
    pub fn unpack<W: Write>(&mut self, out: W) -> Result<u64, ReadError>
    {
        unpack_object(self.container, self.header, out)
    }

    /// Loads the name of this object if it's not already loaded.
    ///
    /// # Errors
    ///
    /// If the name is not already loaded, returns a [ReadError](crate::package::error::ReadError)
    /// if the section couldn't be loaded or the string couldn't be loaded.
    pub fn load_name(&mut self) -> Result<&str, ReadError>
    {
        load_string_section(self.container, self.strings)?;
        let name = self.strings.get(self.container, self.header.name)?;
        Ok(name)
    }

    /// Returns the size in bytes of this object.
    pub fn size(&self) -> u64
    {
        self.header.size
    }
}

/// An iterator over [Object](crate::package::Object).
pub struct ObjectIter<'a, T>
{
    container: &'a mut Container<T>,
    strings: &'a mut StringSection,
    iter: Iter<'a, ObjectHeader>
}

impl<'a, T> Iterator for ObjectIter<'a, T>
{
    type Item = Object<'a, T>;

    fn next(&mut self) -> Option<Self::Item>
    {
        let header = self.iter.next()?;
        unsafe {
            let ptr = self.container as *mut Container<T>;
            let ptr1 = self.strings as *mut StringSection;
            Some(Object {
                header,
                strings: &mut *ptr1,
                container: &mut *ptr
            })
        }
    }
}

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
/// bpxp.pack("TestObject", "This is a test 你好".as_bytes());
/// bpxp.save().unwrap();
/// //Reset our bytebuf pointer to start
/// let mut bytebuf = bpxp.into_inner().into_inner();
/// bytebuf.seek(SeekFrom::Start(0)).unwrap();
/// //Attempt decoding our in-memory BPXP
/// let mut bpxp = Package::open(bytebuf).unwrap();
/// let items = bpxp.objects().unwrap().count();
/// assert_eq!(items, 1);
/// let mut object = bpxp.objects().unwrap().last().unwrap();
/// assert_eq!(object.load_name().unwrap(), "TestObject");
/// {
///     let mut data = Vec::new();
///     object.unpack(&mut data);
///     let s = std::str::from_utf8(&data).unwrap();
///     assert_eq!(s, "This is a test 你好")
/// }
/// {
///     let mut data = Vec::new();
///     bpxp.unpack("TestObject", &mut data).unwrap();
///     let s = std::str::from_utf8(&data).unwrap();
///     assert_eq!(s, "This is a test 你好")
/// }
/// ```
pub struct Package<T>
{
    settings: Settings,
    container: Container<T>,
    object_table: Handle,
    strings: StringSection,
    objects: Vec<ObjectHeader>,
    table: Option<ItemTable<ObjectHeader>>,
    last_data_section: Option<Handle>
}

impl<T> Package<T>
{
    /// Gets the two bytes of BPXP type.
    pub fn get_type_code(&self) -> [u8; 2]
    {
        self.settings.type_code
    }

    /// Gets the target CPU [Architecture](crate::package::Architecture) for this BPXP.
    pub fn get_architecture(&self) -> Architecture
    {
        self.settings.architecture
    }

    /// Gets the target [Platform](crate::package::Platform) for this BPXP.
    pub fn get_platform(&self) -> Platform
    {
        self.settings.platform
    }

    /// Consumes this Package and returns the inner BPX container.
    pub fn into_inner(self) -> Container<T>
    {
        self.container
    }
}

impl<T: Write + Seek> Package<T>
{
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
        let object_table = container.create_section(
            SectionHeaderBuilder::new()
                .checksum(Checksum::Weak)
                .compression(CompressionMethod::Zlib)
                .ty(SECTION_TYPE_OBJECT_TABLE)
        );
        let string_section = container.create_section(
            SectionHeaderBuilder::new()
                .checksum(Checksum::Weak)
                .compression(CompressionMethod::Zlib)
                .ty(SECTION_TYPE_STRING)
        );
        let strings = StringSection::new(string_section);
        if let Some(metadata) = &settings.metadata {
            let metadata_section = container.create_section(
                SectionHeaderBuilder::new()
                    .checksum(Checksum::Weak)
                    .compression(CompressionMethod::Zlib)
                    .ty(SECTION_TYPE_SD)
            );
            let mut section = container.get_mut(metadata_section);
            metadata.write(section.open().ok_or(WriteError::SectionNotLoaded)?)?;
        }
        Ok(Package {
            settings,
            strings,
            container,
            object_table,
            objects: Vec::new(),
            table: None,
            last_data_section: None
        })
    }

    fn write_object<TRead: Read>(
        &mut self,
        source: &mut TRead,
        data_id: Handle
    ) -> Result<(usize, bool), WriteError>
    {
        let mut section = self.container.get_mut(data_id);
        let data = section.open().ok_or(WriteError::SectionNotLoaded)?;
        let mut buf: [u8; DATA_WRITE_BUFFER_SIZE] = [0; DATA_WRITE_BUFFER_SIZE];
        let mut res = source.read_fill(&mut buf)?;
        let mut count = res;

        while res > 0 {
            data.write_all(&buf[0..res])?;
            if data.size() >= MAX_DATA_SECTION_SIZE
            //Split sections (this is to avoid reaching the 4Gb max)
            {
                return Ok((count, true));
            }
            res = source.read_fill(&mut buf)?;
            count += res;
        }
        Ok((count, false))
    }

    pub fn pack<R: Read>(&mut self, name: &str, mut source: R) -> Result<(), WriteError>
    {
        let mut object_size = 0;
        let mut data_section = *self
            .last_data_section
            .get_or_insert_with(|| self.container.create_section(create_data_section_header()));
        let start = self.container.get(data_section).index();
        let offset = {
            let section = self.container.get(data_section);
            section.open().ok_or(WriteError::SectionNotLoaded)?.size()
        } as u32;

        loop {
            let (count, need_section) = self.write_object(&mut source, data_section)?;
            object_size += count;
            if need_section {
                data_section = self.container.create_section(create_data_section_header());
            } else {
                break;
            }
        }
        {
            // Fill and write the object header
            let buf = ObjectHeader {
                size: object_size as u64,
                name: self.strings.put(&mut self.container, name)?,
                start,
                offset
            };
            self.objects.push(buf);
        }
        {
            let section = self.container.get(data_section);
            if section.open().ok_or(WriteError::SectionNotLoaded)?.size() > MAX_DATA_SECTION_SIZE {
                self.last_data_section = None;
            } else {
                self.last_data_section = Some(data_section);
            }
        }
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), WriteError>
    {
        {
            let mut section = self.container.get_mut(self.object_table);
            let data = section.open().ok_or(WriteError::SectionNotLoaded)?;
            data.seek(SeekFrom::Start(0))?;
            for v in &self.objects {
                v.write(data)?;
            }
        }
        self.container.save()?;
        Ok(())
    }
}

impl<T: Read + Seek> Package<T>
{
    /// Creates a new Package by reading from a BPX decoder.
    ///
    /// # Arguments
    ///
    /// * `backend`: a [Read](std::io::Read) + [Seek](std::io::Seek) to use as backend.
    ///
    /// returns: Result<PackageDecoder<TBackend>, Error>
    ///
    /// # Errors
    ///
    /// A [ReadError](crate::package::error::ReadError) is returned if some
    /// sections/headers could not be loaded.
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
        let strings =
            StringSection::new(match container.find_section_by_type(SECTION_TYPE_STRING) {
                Some(v) => v,
                None => return Err(ReadError::MissingSection(Section::Strings))
            });
        let object_table = match container.find_section_by_type(SECTION_TYPE_OBJECT_TABLE) {
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
            strings,
            object_table,
            container,
            objects: Vec::new(),
            table: None,
            last_data_section: None
        })
    }

    pub fn objects(&mut self) -> Result<ObjectIter<T>, ReadError>
    {
        let table = self.table.get_or_insert_with_err(|| {
            read_object_table(&mut self.container, &mut self.objects, self.object_table)
        })?;
        let iter = table.iter();
        Ok(ObjectIter {
            container: &mut self.container,
            strings: &mut self.strings,
            iter
        })
    }

    /// Removes an object from this package.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the object to remove.
    ///
    /// returns: Result<bool, ReadError>
    pub fn remove(&mut self, name: &str) -> Result<bool, ReadError>
    {
        let mut idx = None;
        for (i, v) in self.objects.iter().enumerate() {
            let name1 = self.strings.get(&mut self.container, v.name)?;
            if name1 == name {
                idx = Some(i);
                break;
            }
        }
        if let Some(i) = idx {
            self.objects.remove(i);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Reads the metadata section of this BPXP if any.
    /// Returns None if there is no metadata in this BPXP.
    ///
    /// # Errors
    ///
    /// A [ReadError](crate::package::error::ReadError) is returned in case of corruption or system error.
    pub fn read_metadata(&mut self) -> Result<Option<crate::sd::Object>, ReadError>
    {
        if let Some(obj) = &self.settings.metadata {
            return Ok(Some(obj.clone()));
        }
        if let Some(handle) = self.container.find_section_by_type(SECTION_TYPE_SD) {
            let mut section = self.container.get_mut(handle);
            let obj = crate::sd::Object::read(section.load()?)?;
            self.settings.metadata = Some(obj.clone());
            return Ok(Some(obj));
        }
        Ok(None)
    }

    /// Unpacks an object and returns the size of the unpacked object or None if the object does not exist.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the object to unpack.
    /// * `out`: the output Write.
    ///
    /// returns: Result<Option<u64>, ReadError>
    pub fn unpack<W: Write>(&mut self, name: &str, out: W) -> Result<Option<u64>, ReadError>
    {
        let table = self.table.get_or_insert_with_err(|| {
            read_object_table(&mut self.container, &mut self.objects, self.object_table)
        })?;
        load_string_section(&mut self.container, &self.strings)?;
        table.build_lookup_table(&mut self.container, &mut self.strings)?;
        if let Some(header) = table.lookup(name) {
            let size = unpack_object(&mut self.container, header, out)?;
            Ok(Some(size))
        } else {
            Ok(None)
        }
    }
}
