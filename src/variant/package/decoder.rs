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
    io::{SeekFrom, Write},
    ops::DerefMut,
    rc::Rc
};

use crate::{
    decoder::{Decoder, IoBackend},
    header::{Struct, SECTION_TYPE_SD, SECTION_TYPE_STRING},
    sd::Object,
    section::AutoSection,
    strings::StringSection,
    variant::{
        package::{
            error::{ReadError, Section},
            object::{ObjectHeader, ObjectTable},
            Architecture,
            Platform,
            SECTION_TYPE_OBJECT_TABLE,
            SUPPORTED_VERSION
        },
        NamedTable
    },
    Interface,
    SectionHandle
};

const DATA_READ_BUFFER_SIZE: usize = 8192;

/// Represents a BPX Package decoder.
pub struct PackageDecoder<TBackend: IoBackend>
{
    type_code: [u8; 2],
    architecture: Architecture,
    platform: Platform,
    strings: StringSection,
    decoder: Decoder<TBackend>,
    object_table: Rc<AutoSection>
}

fn get_arch_platform_from_code(acode: u8, pcode: u8) -> Result<(Architecture, Platform), ReadError>
{
    let arch;
    let platform;

    match acode {
        0x0 => arch = Architecture::X86_64,
        0x1 => arch = Architecture::Aarch64,
        0x2 => arch = Architecture::X86,
        0x3 => arch = Architecture::Armv7hl,
        0x4 => arch = Architecture::Any,
        _ => return Err(ReadError::InvalidArchCode(acode))
    }
    match pcode {
        0x0 => platform = Platform::Linux,
        0x1 => platform = Platform::Mac,
        0x2 => platform = Platform::Windows,
        0x3 => platform = Platform::Android,
        0x4 => platform = Platform::Any,
        _ => return Err(ReadError::InvalidPlatformCode(pcode))
    }
    return Ok((arch, platform));
}

impl<TBackend: IoBackend> PackageDecoder<TBackend>
{
    /// Creates a new PackageDecoder by reading from a BPX decoder.
    ///
    /// # Arguments
    ///
    /// * `backend`: the [IoBackend](crate::decoder::IoBackend) to use.
    ///
    /// returns: Result<PackageDecoder<TBackend>, Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned if some sections/headers could not be loaded.
    pub fn new(backend: TBackend) -> Result<PackageDecoder<TBackend>, ReadError>
    {
        let mut decoder = Decoder::new(backend)?;
        if decoder.get_main_header().btype != 'P' as u8 {
            return Err(ReadError::BadType(decoder.get_main_header().btype));
        }
        if decoder.get_main_header().version != SUPPORTED_VERSION {
            return Err(ReadError::BadVersion(decoder.get_main_header().version));
        }
        let (a, p) = get_arch_platform_from_code(
            decoder.get_main_header().type_ext[0],
            decoder.get_main_header().type_ext[1]
        )?;
        let strings = match decoder.find_section_by_type(SECTION_TYPE_STRING) {
            Some(v) => v,
            None => return Err(ReadError::MissingSection(Section::Strings))
        };
        let object_table = match decoder.find_section_by_type(SECTION_TYPE_OBJECT_TABLE) {
            Some(v) => v,
            None => return Err(ReadError::MissingSection(Section::ObjectTable))
        };
        return Ok(PackageDecoder {
            architecture: a,
            platform: p,
            strings: StringSection::new(decoder.load_section(strings)?.clone()),
            type_code: [
                decoder.get_main_header().type_ext[2],
                decoder.get_main_header().type_ext[3]
            ],
            object_table: decoder.load_section(object_table)?.clone(),
            decoder
        });
    }

    /// Gets the two bytes of BPXP variant.
    pub fn get_variant(&self) -> [u8; 2]
    {
        return self.type_code;
    }

    /// Gets the target CPU [Architecture](crate::variant::package::Architecture) for this BPXP.
    pub fn get_architecture(&self) -> Architecture
    {
        return self.architecture;
    }

    /// Gets the target [Platform](crate::variant::package::Platform) for this BPXP.
    pub fn get_platform(&self) -> Platform
    {
        return self.platform;
    }

    /// Reads the metadata section of this BPXP if any.
    /// Returns None if there is no metadata in this BPXP.
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned in case of corruption or system error.
    pub fn read_metadata(&mut self) -> Result<Option<Object>, ReadError>
    {
        if let Some(handle) = self.decoder.find_section_by_type(SECTION_TYPE_SD) {
            let section = self.decoder.load_section(handle)?;
            let mut data = section.open()?;
            let obj = Object::read(&mut *data)?;
            return Ok(Some(obj));
        }
        return Ok(None);
    }

    /// Reads the object table of this BPXP.
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned in case of corruption or system error.
    pub fn read_object_table(&mut self) -> Result<ObjectTable, ReadError>
    {
        use crate::section::Section;
        let mut v = Vec::new();
        let count = self.object_table.size() / 20;
        let mut object_table = self.object_table.open()?;

        for _ in 0..count {
            //Type inference in Rust is so buggy! One &mut dyn is not enough you need double &mut dyn now!
            let header = ObjectHeader::read(&mut object_table.deref_mut())?;
            v.push(header);
        }
        return Ok(ObjectTable::new(v));
    }

    /// Gets the name of an object; loads the string if its not yet loaded.
    ///
    /// # Arguments
    ///
    /// * `obj`: the object header to load the actual name for.
    ///
    /// returns: Result<&str, Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned if the name could not be read.
    pub fn get_object_name(&mut self, obj: &ObjectHeader) -> Result<&str, crate::strings::ReadError>
    {
        return self.strings.get(obj.name);
    }

    fn load_from_section<TWrite: Write>(
        &mut self,
        handle: SectionHandle,
        offset: u32,
        size: u32,
        out: &mut TWrite
    ) -> Result<u32, ReadError>
    {
        let mut len = 0;
        let mut buf: [u8; DATA_READ_BUFFER_SIZE] = [0; DATA_READ_BUFFER_SIZE];
        let section = self.decoder.load_section(handle)?;
        let mut data = section.open()?;

        data.seek(SeekFrom::Start(offset as u64))?;
        while len < size {
            let s = std::cmp::min(size - len, DATA_READ_BUFFER_SIZE as u32);
            let val = data.read(&mut buf[0..s as usize])?;
            len += val as u32;
            out.write(&buf[0..val])?;
        }
        return Ok(len);
    }

    /// Unpacks an object to a raw stream.
    /// Returns the number of bytes read if the operation has succeeded.
    ///
    /// # Arguments
    ///
    /// * `obj`: the object header.
    /// * `out`: the raw [Write](std::io::Write) to use as destination.
    ///
    /// returns: Result<u64, Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned if the object could not be unpacked.
    pub fn unpack_object<TWrite: Write>(&mut self, obj: &ObjectHeader, mut out: TWrite) -> Result<u64, ReadError>
    {
        let mut section_id = obj.start;
        let mut offset = obj.offset;
        let mut len = obj.size;

        while len > 0 {
            let handle = match self.decoder.find_section_by_index(section_id) {
                Some(i) => i,
                None => break
            };
            let remaining_section_size = self.decoder.get_section_header(handle).size - offset;
            let val = self.load_from_section(
                handle,
                offset,
                std::cmp::min(remaining_section_size as u64, len) as u32,
                &mut out
            )?;
            len -= val as u64;
            offset = 0;
            section_id += 1;
        }
        return Ok(obj.size);
    }

    /// Consumes this BPXP decoder and returns the inner BPX decoder.
    pub fn into_inner(self) -> Decoder<TBackend>
    {
        return self.decoder;
    }
}
