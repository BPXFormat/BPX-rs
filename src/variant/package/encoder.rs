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

use std::io::Read;

use byteorder::{ByteOrder, LittleEndian};

use crate::{
    builder::{Checksum, CompressionMethod, MainHeaderBuilder, SectionHeaderBuilder},
    encoder::{Encoder, IoBackend},
    header::{SectionHeader, SECTION_TYPE_SD, SECTION_TYPE_STRING},
    sd::Object,
    strings::StringSection,
    utils::OptionExtension,
    variant::package::{Architecture, Platform, SECTION_TYPE_DATA, SECTION_TYPE_OBJECT_TABLE},
    Interface,
    Result,
    SectionHandle
};
use crate::variant::package::SUPPORTED_VERSION;

const DATA_WRITE_BUFFER_SIZE: usize = 8192;
const MIN_DATA_REMAINING_SIZE: usize = DATA_WRITE_BUFFER_SIZE;
const MAX_DATA_SECTION_SIZE: usize = 200000000 - MIN_DATA_REMAINING_SIZE; //200MB

/// Utility to easily generate a [PackageEncoder](crate::variant::package::PackageEncoder)
pub struct PackageBuilder
{
    architecture: Architecture,
    platform: Platform,
    metadata: Option<Object>,
    type_code: [u8; 2]
}

impl PackageBuilder
{
    /// Creates a new BPX Package builder
    ///
    /// # Returns
    ///
    /// * the new BPX Package builder
    pub fn new() -> PackageBuilder
    {
        return PackageBuilder {
            architecture: Architecture::Any,
            platform: Platform::Any,
            metadata: None,
            type_code: [0x50, 0x48]
        };
    }

    /// Defines the CPU architecture that the package is targeting
    ///
    /// - *By default, no CPU architecture is targeted*
    ///
    /// # Arguments
    ///
    /// * `arch` - the new [Architecture](crate::variant::package::Architecture)
    pub fn with_architecture(mut self, arch: Architecture) -> Self
    {
        self.architecture = arch;
        return self;
    }

    /// Defines the platform that the package is targeting
    ///
    /// - *By default, no platform is targeted*
    ///
    /// # Arguments
    ///
    /// * `platform` - the new [Platform](crate::variant::package::Platform)
    pub fn with_platform(mut self, platform: Platform) -> Self
    {
        self.platform = platform;
        return self;
    }

    /// Defines the metadata for the package
    ///
    /// - *By default, no metadata object is set*
    ///
    /// # Arguments
    ///
    /// * `obj` - the new BPXSD [Object](crate::sd::Object) metadata
    pub fn with_metadata(mut self, obj: Object) -> Self
    {
        self.metadata = Some(obj);
        return self;
    }

    /// Defines the variant of the package
    ///
    /// - *By default, the package variant is 'PK' to identify a package destined for FPKG C++ package manager*
    ///
    /// # Arguments
    ///
    /// * `type_code` - an array with 2 bytes
    pub fn with_variant(mut self, type_code: [u8; 2]) -> Self
    {
        self.type_code = type_code;
        return self;
    }

    /// Builds the corresponding [PackageEncoder](crate::variant::package::PackageEncoder)
    ///
    /// # Arguments
    ///
    /// * `encoder` - the BPX [Encoder](crate::encoder::Encoder) backend to use
    ///
    /// # Returns
    ///
    /// * the new [PackageEncoder](crate::variant::package::PackageEncoder) if the operation succeeded
    /// * an [Error](crate::error::Error) in case of system error
    pub fn build<TBackend: IoBackend>(self, encoder: &mut Encoder<TBackend>) -> Result<PackageEncoder<TBackend>>
    {
        let mut type_ext: [u8; 16] = [0; 16];
        match self.architecture {
            Architecture::X86_64 => type_ext[0] = 0x0,
            Architecture::Aarch64 => type_ext[0] = 0x1,
            Architecture::X86 => type_ext[0] = 0x2,
            Architecture::Armv7hl => type_ext[0] = 0x3,
            Architecture::Any => type_ext[0] = 0x4
        }
        match self.platform {
            Platform::Linux => type_ext[1] = 0x0,
            Platform::Mac => type_ext[1] = 0x1,
            Platform::Windows => type_ext[1] = 0x2,
            Platform::Android => type_ext[1] = 0x3,
            Platform::Any => type_ext[1] = 0x4
        }
        type_ext[2] = self.type_code[0];
        type_ext[3] = self.type_code[1];
        let header = MainHeaderBuilder::new()
            .with_type('P' as u8)
            .with_type_ext(type_ext)
            .with_version(SUPPORTED_VERSION)
            .build();
        encoder.set_main_header(header);
        let strings_header = SectionHeaderBuilder::new()
            .with_checksum(Checksum::Weak)
            .with_compression(CompressionMethod::Zlib)
            .with_type(SECTION_TYPE_STRING)
            .build();
        let object_table_header = SectionHeaderBuilder::new()
            .with_checksum(Checksum::Weak)
            .with_compression(CompressionMethod::Zlib)
            .with_type(SECTION_TYPE_OBJECT_TABLE)
            .build();
        let strings = encoder.create_section(strings_header)?;
        let object_table = encoder.create_section(object_table_header)?;
        if let Some(obj) = self.metadata {
            let metadata_header = SectionHeaderBuilder::new()
                .with_checksum(Checksum::Weak)
                .with_compression(CompressionMethod::Zlib)
                .with_type(SECTION_TYPE_SD)
                .build();
            let metadata = encoder.create_section(metadata_header)?;
            obj.write(&mut encoder.open_section(metadata)?)?;
        }
        return Ok(PackageEncoder {
            strings,
            encoder,
            last_data_section: None,
            object_table
        });
    }
}

/// Represents a BPX Package encoder
pub struct PackageEncoder<'a, TBackend: IoBackend>
{
    strings: SectionHandle,
    last_data_section: Option<SectionHandle>,
    object_table: SectionHandle,
    encoder: &'a mut Encoder<TBackend>
}

fn create_data_section_header() -> SectionHeader
{
    let header = SectionHeaderBuilder::new()
        .with_type(SECTION_TYPE_DATA)
        .with_compression(CompressionMethod::Xz)
        .with_checksum(Checksum::Crc32)
        .build();
    return header;
}

impl<'a, TBackend: IoBackend> PackageEncoder<'a, TBackend>
{
    fn write_object<TRead: Read>(&mut self, source: &mut TRead, data_id: SectionHandle) -> Result<(usize, bool)>
    {
        let data = self.encoder.open_section(data_id)?;
        let mut buf: [u8; DATA_WRITE_BUFFER_SIZE] = [0; DATA_WRITE_BUFFER_SIZE];
        let mut res = source.read(&mut buf)?;
        let mut count = res;

        while res > 0 {
            data.write(&buf[0..res])?;
            if data.size() >= MAX_DATA_SECTION_SIZE
            //Split sections (this is to avoid reaching the 4Gb max)
            {
                return Ok((count, false));
            }
            res = source.read(&mut buf)?;
            count += res;
        }
        return Ok((count, true));
    }

    /// Stores an object in this BPXP with the given name
    ///
    /// *this functions prints some information to standard output as a way to debug data compression issues*
    ///
    /// # Arguments
    ///
    /// * `name` - the name of the object
    /// * `source` - the source object data as a [Read](std::io::Read)
    ///
    /// # Returns
    ///
    /// * nothing if the operation succeeded
    /// * an [Error](crate::error::Error) in case of system error
    pub fn pack_object<TRead: Read>(&mut self, name: &str, source: &mut TRead) -> Result<()>
    {
        let mut object_size = 0;
        let useless = &mut self.encoder;
        let mut data_section = *Option::get_or_insert_with_err(&mut self.last_data_section, || {
            useless.create_section(create_data_section_header())
        })?;
        let start = self.encoder.get_section_index(data_section);
        let offset = self.encoder.open_section(data_section)?.size() as u32;

        loop {
            let (count, need_section) = self.write_object(source, data_section)?;
            object_size += count;
            if need_section {
                data_section = self.encoder.create_section(create_data_section_header())?;
            } else {
                break;
            }
        }
        {
            // Fill and write the object header
            let mut buf: [u8; 20] = [0; 20];
            let mut strings = StringSection::new(self.strings);
            LittleEndian::write_u64(&mut buf[0..8], object_size as u64);
            LittleEndian::write_u32(&mut buf[8..12], strings.put(self.encoder, &name)?);
            LittleEndian::write_u32(&mut buf[12..16], start);
            LittleEndian::write_u32(&mut buf[16..20], offset);
            // Write the object header
            let object_table = self.encoder.open_section(self.object_table)?;
            object_table.write(&buf)?;
        }
        if self.encoder.open_section(data_section)?.size() > MAX_DATA_SECTION_SIZE {
            self.last_data_section = None;
        } else {
            self.last_data_section = Some(data_section);
        }
        return Ok(());
    }
}
