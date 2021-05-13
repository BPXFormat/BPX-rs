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

use std::string::String;
use std::path::Path;
use std::fs::metadata;
use std::io::Read;
use std::fs::File;
use std::fs::read_dir;
use byteorder::LittleEndian;
use byteorder::ByteOrder;

use crate::bpxp::Architecture;
use crate::bpxp::Platform;
use crate::sd::Object;
use crate::Result;
use crate::encoder::IoBackend;
use crate::encoder::Encoder;
use crate::SectionHandle;
use crate::builder::SectionHeaderBuilder;
use crate::builder::MainHeaderBuilder;
use crate::builder::Checksum;
use crate::builder::CompressionMethod;
use crate::header::SECTION_TYPE_STRING;
use crate::header::SECTION_TYPE_SD;
use crate::Interface;
use crate::strings::StringSection;
use crate::strings::get_name_from_dir_entry;
use crate::strings::get_name_from_path;
use crate::header::SectionHeader;
use crate::bpxp::DATA_SECTION_TYPE;

const DATA_WRITE_BUFFER_SIZE: usize = 8192;
const MIN_DATA_REMAINING_SIZE: usize = DATA_WRITE_BUFFER_SIZE;
const MAX_DATA_SECTION_SIZE: usize = 200000000 - MIN_DATA_REMAINING_SIZE; //200MB

pub struct PackageBuilder
{
    architecture: Architecture,
    platform: Platform,
    metadata: Option<Object>,
    type_code: [u8; 2]
}

impl PackageBuilder
{
    pub fn with_architecture(mut self, arch: Architecture) -> Self
    {
        self.architecture = arch;
        return self;
    }

    pub fn with_platform(mut self, platform: Platform) -> Self
    {
        self.platform = platform;
        return self;
    }

    pub fn with_metadata(mut self, obj: Object) -> Self
    {
        self.metadata = Some(obj);
        return self;
    }

    pub fn with_variant(mut self, type_code: [u8; 2]) -> Self
    {
        self.type_code = type_code;
        return self;
    }

    pub fn build<'a, TBackend: IoBackend>(self, encoder: &mut Encoder<TBackend>) -> Result<PackageEncoder>
    {
        let mut type_ext: [u8; 16] = [0; 16];
        match self.architecture
        {
            Architecture::X86_64 => type_ext[0] = 0x0,
            Architecture::Aarch64 => type_ext[0] = 0x1,
            Architecture::X86 => type_ext[0] = 0x2,
            Architecture::Armv7hl => type_ext[0] = 0x3,
            Architecture::Any => type_ext[0] = 0x4,
        }
        match self.platform
        {
            Platform::Linux => type_ext[1] = 0x0,
            Platform::Mac => type_ext[1] = 0x1,
            Platform::Windows => type_ext[1] = 0x2,
            Platform::Android => type_ext[1] = 0x3,
            Platform::Any => type_ext[1] = 0x4,
        }
        type_ext[2] = self.type_code[0];
        type_ext[3] = self.type_code[1];
        let header = MainHeaderBuilder::new()
                        .with_type('P' as u8)
                        .with_type_ext(type_ext)
                        .build();
        encoder.set_main_header(header);
        let strings_header = SectionHeaderBuilder::new()
                                .with_checksum(Checksum::Weak)
                                .with_compression(CompressionMethod::Xz) //TODO: replace by Zlib when available
                                .with_type(SECTION_TYPE_STRING)
                                .build();
        let strings = encoder.create_section(strings_header)?;
        if let Some(obj) = self.metadata
        {
            let metadata_header = SectionHeaderBuilder::new()
                                    .with_checksum(Checksum::Weak)
                                    .with_compression(CompressionMethod::Xz) //TODO: replace by Zlib when available
                                    .with_type(SECTION_TYPE_SD)
                                    .build();
            let metadata = encoder.create_section(metadata_header)?;
            obj.write(&mut encoder.open_section(metadata)?)?;
        }
        return Ok(PackageEncoder
        {
            strings: strings
        });
    }
}

pub struct PackageEncoder
{
    strings: SectionHandle
}

fn create_data_section_header() -> SectionHeader
{
    let header = SectionHeaderBuilder::new()
                    .with_type(DATA_SECTION_TYPE)
                    .with_compression(CompressionMethod::Xz)
                    .with_checksum(Checksum::Weak) //TODO: change to Crc32 when available
                    .build();
    return header;
}

impl PackageEncoder
{
    fn write_file<TBackend: IoBackend>(&self, encoder: &mut Encoder<TBackend>, source: &mut dyn Read, data_id: SectionHandle) -> Result<bool>
    {
        let data = encoder.open_section(data_id)?;
        let mut buf: [u8; DATA_WRITE_BUFFER_SIZE] = [0; DATA_WRITE_BUFFER_SIZE];
        let mut res = source.read(&mut buf)?;

        while res > 0
        {
            data.write(&buf[0..res])?;
            if data.size() >= MAX_DATA_SECTION_SIZE //Split sections (this is to avoid reaching the 4Gb max)
            {
                return Ok(false);
            }
            res = source.read(&mut buf)?;
        }
        return Ok(true);
    }

    fn pack_file<TBackend: IoBackend>(&self, encoder: &mut Encoder<TBackend>, source: &Path, name: String, data_id1: SectionHandle, strings: &mut StringSection) -> Result<SectionHandle>
    {
        let mut data_id = data_id1;
        let size = metadata(source)?.len();
        let mut fle = File::open(source)?;
        let mut buf: [u8; 12] = [0; 12];

        println!("Writing file {} with {} byte(s)", name, size);
        LittleEndian::write_u64(&mut buf[0..8], size);
        LittleEndian::write_u32(&mut buf[8..12], strings.put(encoder, &name)?);
        {
            let data = encoder.open_section(data_id)?;
            data.write(&buf)?;
        }
        while !self.write_file(encoder, &mut fle, data_id)?
        {
            data_id = encoder.create_section(create_data_section_header())?;
        }
        return Ok(data_id);
    }

    fn pack_dir<TBackend: IoBackend>(&self, encoder: &mut Encoder<TBackend>, source: &Path, name: String, data_id1: SectionHandle, strings: &mut StringSection) -> Result<()>
    {
        let mut data_id = data_id1;
        let entries = read_dir(source)?;
    
        for rentry in entries
        {
            let entry = rentry?;
            let mut s = name.clone();
            s.push('/');
            s.push_str(&get_name_from_dir_entry(&entry));
            if entry.file_type()?.is_dir()
            {
                self.pack_dir(encoder, &entry.path(), s, data_id, strings)?
            }
            else
            {
                data_id = self.pack_file(encoder, &entry.path(), s, data_id, strings)?;
            }
        }
        return Ok(());
    }

    pub fn pack_vname<TBackend: IoBackend>(&self, encoder: &mut Encoder<TBackend>, source: &Path, vname: &str) -> Result<()>
    {
        let mut strings = StringSection::new(self.strings);
        let md = metadata(source)?;
        let data_section = encoder.create_section(create_data_section_header())?;
        if md.is_file()
        {
            self.pack_file(encoder, source, String::from(vname), data_section, &mut strings)?;
            return Ok(());
        }
        else
        {
            return self.pack_dir(encoder, source, String::from(vname), data_section, &mut strings);
        }
    }
    
    pub fn pack<TBackend: IoBackend>(&self, encoder: &mut Encoder<TBackend>, source: &Path) -> Result<()>
    {
        let mut strings = StringSection::new(self.strings);
        let md = metadata(source)?;
        let data_section = encoder.create_section(create_data_section_header())?;
        if md.is_file()
        {
            self.pack_file(encoder, source, get_name_from_path(source)?, data_section, &mut strings)?;
            return Ok(());
        }
        else
        {
            return self.pack_dir(encoder, source, get_name_from_path(source)?, data_section, &mut strings);
        }
    }
}
