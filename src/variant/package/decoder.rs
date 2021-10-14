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

use std::io::{SeekFrom, Write};

use byteorder::{ByteOrder, LittleEndian};

use crate::{
    decoder::{Decoder, IoBackend},
    error::Error,
    header::{SECTION_TYPE_SD, SECTION_TYPE_STRING},
    sd::Object,
    strings::StringSection,
    variant::package::{
        object::{ObjectHeader, ObjectTable},
        Architecture,
        Platform,
        SECTION_TYPE_OBJECT_TABLE,
        SUPPORTED_VERSION
    },
    Interface,
    Result,
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
    object_table: SectionHandle
}

fn get_arch_platform_from_code(acode: u8, pcode: u8) -> Result<(Architecture, Platform)>
{
    let arch;
    let platform;

    match acode {
        0x0 => arch = Architecture::X86_64,
        0x1 => arch = Architecture::Aarch64,
        0x2 => arch = Architecture::X86,
        0x3 => arch = Architecture::Armv7hl,
        0x4 => arch = Architecture::Any,
        _ => return Err(Error::Corruption(String::from("Architecture code does not exist")))
    }
    match pcode {
        0x0 => platform = Platform::Linux,
        0x1 => platform = Platform::Mac,
        0x2 => platform = Platform::Windows,
        0x3 => platform = Platform::Android,
        0x4 => platform = Platform::Any,
        _ => return Err(Error::Corruption(String::from("Platform code does not exist")))
    }
    return Ok((arch, platform));
}

impl<TBackend: IoBackend> PackageDecoder<TBackend>
{
    /// Creates a new PackageDecoder by reading from a BPX decoder.
    ///
    /// # Arguments
    ///
    /// * `decoder`: the BPX [Decoder](crate::decoder::Decoder) backend to use.
    ///
    /// returns: Result<PackageDecoder<TBackend>, Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned if some sections/headers could not be loaded.
    pub fn new(backend: TBackend) -> Result<PackageDecoder<TBackend>>
    {
        let decoder = Decoder::new(backend)?;
        if decoder.get_main_header().btype != 'P' as u8 {
            return Err(Error::Corruption(format!(
                "Unknown variant of BPX: {}",
                decoder.get_main_header().btype as char
            )));
        }
        if decoder.get_main_header().version != SUPPORTED_VERSION {
            return Err(Error::Unsupported(format!(
                "This version of the BPX SDK only supports BPXP version {}, you are trying to decode version {} BPXP",
                SUPPORTED_VERSION,
                decoder.get_main_header().version
            )));
        }
        let (a, p) = get_arch_platform_from_code(
            decoder.get_main_header().type_ext[0],
            decoder.get_main_header().type_ext[1]
        )?;
        let strings = match decoder.find_section_by_type(SECTION_TYPE_STRING) {
            Some(v) => v,
            None => return Err(Error::Corruption(String::from("Unable to locate strings section")))
        };
        let object_table = match decoder.find_section_by_type(SECTION_TYPE_OBJECT_TABLE) {
            Some(v) => v,
            None => return Err(Error::Corruption(String::from("Unable to locate BPXP object table")))
        };
        return Ok(PackageDecoder {
            architecture: a,
            platform: p,
            strings: StringSection::new(strings),
            type_code: [
                decoder.get_main_header().type_ext[2],
                decoder.get_main_header().type_ext[3]
            ],
            decoder,
            object_table
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
    pub fn read_metadata(&mut self) -> Result<Option<Object>>
    {
        if let Some(handle) = self.decoder.find_section_by_type(SECTION_TYPE_SD) {
            let mut data = self.decoder.open_section(handle)?;
            let obj = Object::read(&mut data)?;
            return Ok(Some(obj));
        }
        return Ok(None);
    }

    /// Reads the object table of this BPXP.
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned in case of corruption or system error.
    pub fn read_object_table(&mut self) -> Result<ObjectTable>
    {
        let mut v = Vec::new();
        let count = self.decoder.get_section_header(self.object_table).size / 20;
        let object_table = self.decoder.open_section(self.object_table)?;

        for _ in 0..count {
            let mut buf: [u8; 20] = [0; 20];
            if object_table.read(&mut buf)? != 20 {
                return Err(Error::Truncation("read object table"));
            }
            let size = LittleEndian::read_u64(&buf[0..8]);
            let name_ptr = LittleEndian::read_u32(&buf[8..12]);
            let start = LittleEndian::read_u32(&buf[12..16]);
            let offset = LittleEndian::read_u32(&buf[16..20]);
            v.push(ObjectHeader {
                size,
                name: name_ptr,
                start,
                offset
            })
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
    pub fn get_object_name(&mut self, obj: &ObjectHeader) -> Result<&str>
    {
        return self.strings.get(&mut self.decoder, obj.name);
    }

    fn load_from_section<TWrite: Write>(
        &mut self,
        handle: SectionHandle,
        offset: u32,
        size: u32,
        out: &mut TWrite
    ) -> Result<u32>
    {
        let mut len = 0;
        let mut buf: [u8; DATA_READ_BUFFER_SIZE] = [0; DATA_READ_BUFFER_SIZE];
        let data = self.decoder.open_section(handle)?;

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
    pub fn unpack_object<TWrite: Write>(&mut self, obj: &ObjectHeader, mut out: TWrite) -> Result<u64>
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
