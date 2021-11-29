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
use std::slice::Iter;
use crate::core::builder::{Checksum, CompressionMethod, MainHeaderBuilder, SectionHeaderBuilder};
use crate::core::{Container, SectionData};
use crate::core::header::{SECTION_TYPE_SD, SECTION_TYPE_STRING, SectionHeader, Struct};
use crate::Handle;
use crate::package::object::ObjectHeader;
use crate::package::{Architecture, Platform, SECTION_TYPE_DATA, SECTION_TYPE_OBJECT_TABLE, Settings, SUPPORTED_VERSION};
use crate::package::error::{InvalidCodeContext, ReadError, Section, WriteError};
use crate::strings::{StringSection};
use crate::table::ItemTable;
use crate::utils::{OptionExtension, ReadFill};

const DATA_READ_BUFFER_SIZE: usize = 8192;
const DATA_WRITE_BUFFER_SIZE: usize = 8192;
const MIN_DATA_REMAINING_SIZE: usize = DATA_WRITE_BUFFER_SIZE;
const MAX_DATA_SECTION_SIZE: usize = 200000000 - MIN_DATA_REMAINING_SIZE; //200MB

pub struct Object<'a, T>
{
    container: &'a mut Container<T>,
    strings: &'a mut StringSection,
    header: &'a ObjectHeader
}

impl<'a, T: Read + Seek> Object<'a, T>
{
    pub fn unpack<W: Write>(&mut self, out: W) -> Result<u64, ReadError>
    {
        unpack_object(self.container, self.header, out)
    }

    pub fn load_name(&mut self) -> Result<&str, ReadError>
    {
        load_string_section(self.container, self.strings)?;
        let name = self.strings.get(self.container, self.header.name)?;
        Ok(name)
    }

    pub fn size(&self) -> u64
    {
        self.header.size
    }
}

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

fn load_string_section<T: Read + Seek>(container: &mut Container<T>, strings: &StringSection) -> Result<(), ReadError>
{
    let mut section = container.get_mut(strings.handle());
    section.load()?;
    Ok(())
}

fn create_data_section_header() -> SectionHeader
{
    SectionHeaderBuilder::new()
        .with_type(SECTION_TYPE_DATA)
        .with_compression(CompressionMethod::Xz)
        .with_checksum(Checksum::Crc32)
        .build()
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
    /// Gets the two bytes of BPXP variant.
    pub fn get_variant(&self) -> [u8; 2]
    {
        self.settings.type_code
    }

    /// Gets the target CPU [Architecture](crate::variant::package::Architecture) for this BPXP.
    pub fn get_architecture(&self) -> Architecture
    {
        self.settings.architecture
    }

    /// Gets the target [Platform](crate::variant::package::Platform) for this BPXP.
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
        let mut type_ext: [u8; 16] = [0; 16];
        match settings.architecture {
            Architecture::X86_64 => type_ext[0] = 0x0,
            Architecture::Aarch64 => type_ext[0] = 0x1,
            Architecture::X86 => type_ext[0] = 0x2,
            Architecture::Armv7hl => type_ext[0] = 0x3,
            Architecture::Any => type_ext[0] = 0x4
        }
        match settings.platform {
            Platform::Linux => type_ext[1] = 0x0,
            Platform::Mac => type_ext[1] = 0x1,
            Platform::Windows => type_ext[1] = 0x2,
            Platform::Android => type_ext[1] = 0x3,
            Platform::Any => type_ext[1] = 0x4
        }
        type_ext[2] = settings.type_code[0];
        type_ext[3] = settings.type_code[1];
        let mut container = Container::create(backend, MainHeaderBuilder::new()
            .with_type(b'P')
            .with_type_ext(type_ext)
            .with_version(SUPPORTED_VERSION));
        let object_table = container.create_section(SectionHeaderBuilder::new()
            .with_checksum(Checksum::Weak)
            .with_compression(CompressionMethod::Zlib)
            .with_type(SECTION_TYPE_OBJECT_TABLE));
        let string_section = container.create_section(SectionHeaderBuilder::new()
            .with_checksum(Checksum::Weak)
            .with_compression(CompressionMethod::Zlib)
            .with_type(SECTION_TYPE_STRING));
        let strings = StringSection::new(string_section);
        if let Some(metadata) = &settings.metadata {
            let metadata_section = container.create_section(SectionHeaderBuilder::new()
                .with_checksum(Checksum::Weak)
                .with_compression(CompressionMethod::Zlib)
                .with_type(SECTION_TYPE_SD));
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
        let mut data_section = *self.last_data_section.get_or_insert_with(|| self.container.create_section(create_data_section_header()));
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

fn get_arch_platform_from_code(acode: u8, pcode: u8)
                               -> Result<(Architecture, Platform), ReadError>
{
    let arch;
    let platform;

    match acode {
        0x0 => arch = Architecture::X86_64,
        0x1 => arch = Architecture::Aarch64,
        0x2 => arch = Architecture::X86,
        0x3 => arch = Architecture::Armv7hl,
        0x4 => arch = Architecture::Any,
        _ => return Err(ReadError::InvalidCode(InvalidCodeContext::Arch, acode))
    }
    match pcode {
        0x0 => platform = Platform::Linux,
        0x1 => platform = Platform::Mac,
        0x2 => platform = Platform::Windows,
        0x3 => platform = Platform::Android,
        0x4 => platform = Platform::Any,
        _ => return Err(ReadError::InvalidCode(InvalidCodeContext::Platform, pcode))
    }
    Ok((arch, platform))
}

fn load_from_section<T: Read + Seek, W: Write>(
    container: &mut Container<T>,
    handle: Handle,
    offset: u32,
    size: u32,
    out: &mut W
) -> Result<u32, ReadError>
{
    let mut len = 0;
    let mut buf: [u8; DATA_READ_BUFFER_SIZE] = [0; DATA_READ_BUFFER_SIZE];
    let mut section = container.get_mut(handle);
    let data = section.load()?;

    data.seek(SeekFrom::Start(offset as u64))?;
    while len < size {
        let s = std::cmp::min(size - len, DATA_READ_BUFFER_SIZE as u32);
        // Read is enough as Sections are guaranteed to fill the buffer as much as possible
        let val = data.read(&mut buf[0..s as usize])?;
        len += val as u32;
        out.write_all(&buf[0..val])?;
    }
    Ok(len)
}

fn unpack_object<T: Read + Seek, W: Write>(container: &mut Container<T>, obj: &ObjectHeader, mut out: W) -> Result<u64, ReadError>
{
    let mut section_id = obj.start;
    let mut offset = obj.offset;
    let mut len = obj.size;

    while len > 0 {
        let handle = match container.find_section_by_index(section_id) {
            Some(i) => i,
            None => break
        };
        let section = container.get(handle);
        let remaining_section_size = section.header().size - offset;
        let val = load_from_section(
            container,
            handle,
            offset,
            std::cmp::min(remaining_section_size as u64, len) as u32,
            &mut out
        )?;
        len -= val as u64;
        offset = 0;
        section_id += 1;
    }
    Ok(obj.size)
}

fn read_object_table<T: Read + Seek>(container: &mut Container<T>, object_table: Handle) -> Result<Vec<ObjectHeader>, ReadError>
{
    let mut section = container.get_mut(object_table);
    let count = section.header().size / 20;
    let mut v = Vec::with_capacity(count as _);

    for _ in 0..count {
        let header = ObjectHeader::read(section.load()?)?;
        v.push(header);
    }
    Ok(v)
}

impl<T: Read + Seek> Package<T>
{
    /// Creates a new Package by reading from a BPX decoder.
    ///
    /// # Arguments
    ///
    /// * `backend`: the [IoBackend](crate::decoder::IoBackend) to use.
    ///
    /// returns: Result<PackageDecoder<TBackend>, Error>
    ///
    /// # Errors
    ///
    /// A [ReadError](crate::variant::package::error::ReadError) is returned if some
    /// sections/headers could not be loaded.
    pub fn open(backend: T) -> Result<Package<T>, ReadError>
    {
        let container = Container::open(backend)?;
        if container.get_main_header().btype != b'P' {
            return Err(ReadError::BadType(container.get_main_header().btype));
        }
        if container.get_main_header().version != SUPPORTED_VERSION {
            return Err(ReadError::BadVersion(container.get_main_header().version));
        }
        let (a, p) = get_arch_platform_from_code(
            container.get_main_header().type_ext[0],
            container.get_main_header().type_ext[1]
        )?;
        let strings = StringSection::new(match container.find_section_by_type(SECTION_TYPE_STRING) {
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
        let table = self.table.get_or_insert_with_err(|| -> Result<ItemTable<ObjectHeader>, ReadError> {
            let v = read_object_table(&mut self.container, self.object_table)?;
            self.objects = v.clone();
            Ok(ItemTable::new(v))
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
    /// A [ReadError](crate::variant::package::error::ReadError) is returned in case of corruption or system error.
    pub fn read_metadata(&mut self) -> Result<Option<crate::sd::Object>, ReadError>
    {
        if let Some(obj) = &self.settings.metadata {
            return Ok(Some(obj.clone()))
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
        let table = self.table.get_or_insert_with_err(|| -> Result<ItemTable<ObjectHeader>, ReadError> {
            let v = read_object_table(&mut self.container, self.object_table)?;
            self.objects = v.clone();
            Ok(ItemTable::new(v))
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
