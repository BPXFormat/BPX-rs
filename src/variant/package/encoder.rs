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

use std::{io::Read, rc::Rc};

use crate::{
    builder::{Checksum, CompressionMethod, MainHeaderBuilder, SectionHeaderBuilder},
    encoder::{Encoder, IoBackend},
    header::{SectionHeader, Struct, SECTION_TYPE_SD, SECTION_TYPE_STRING},
    sd::Object,
    section::{AutoSection, Section},
    strings::StringSection,
    utils::OptionExtension,
    variant::package::{
        error::WriteError,
        object::ObjectHeader,
        Architecture,
        Platform,
        SECTION_TYPE_DATA,
        SECTION_TYPE_OBJECT_TABLE,
        SUPPORTED_VERSION
    },
    Interface
};
use crate::utils::ReadFill;

const DATA_WRITE_BUFFER_SIZE: usize = 8192;
const MIN_DATA_REMAINING_SIZE: usize = DATA_WRITE_BUFFER_SIZE;
const MAX_DATA_SECTION_SIZE: usize = 200000000 - MIN_DATA_REMAINING_SIZE; //200MB

/// Utility to easily generate a [PackageEncoder](crate::variant::package::PackageEncoder).
pub struct PackageBuilder
{
    architecture: Architecture,
    platform: Platform,
    metadata: Option<Object>,
    type_code: [u8; 2]
}

impl PackageBuilder
{
    /// Creates a new BPX Package builder.
    pub fn new() -> PackageBuilder
    {
        return PackageBuilder {
            architecture: Architecture::Any,
            platform: Platform::Any,
            metadata: None,
            type_code: [0x50, 0x48]
        };
    }

    /// Defines the CPU architecture that the package is targeting.
    ///
    /// *By default, no CPU architecture is targeted.*
    ///
    /// # Arguments
    ///
    /// * `arch`:
    ///
    /// returns: PackageBuilder
    pub fn with_architecture(mut self, arch: Architecture) -> Self
    {
        self.architecture = arch;
        return self;
    }

    /// Defines the platform that the package is targeting.
    ///
    /// *By default, no platform is targeted.*
    ///
    /// # Arguments
    ///
    /// * `platform`:
    ///
    /// returns: PackageBuilder
    pub fn with_platform(mut self, platform: Platform) -> Self
    {
        self.platform = platform;
        return self;
    }

    /// Defines the metadata for the package.
    ///
    /// *By default, no metadata object is set.*
    ///
    /// # Arguments
    ///
    /// * `obj`:
    ///
    /// returns: PackageBuilder
    pub fn with_metadata(mut self, obj: Object) -> Self
    {
        self.metadata = Some(obj);
        return self;
    }

    /// Defines the type of the package.
    ///
    /// *By default, the package variant is 'PK' to identify
    /// a package designed for FPKG.*
    ///
    /// # Arguments
    ///
    /// * `type_code`:
    ///
    /// returns: PackageBuilder
    pub fn with_type(mut self, type_code: [u8; 2]) -> Self
    {
        self.type_code = type_code;
        return self;
    }

    /// Builds the corresponding [PackageEncoder](crate::variant::package::PackageEncoder).
    ///
    /// # Arguments
    ///
    /// * `backend`: the [IoBackend](crate::encoder::IoBackend) to use.
    ///
    /// returns: Result<PackageEncoder<TBackend>, Error>
    ///
    /// # Errors
    ///
    /// A [WriteError](crate::variant::package::error::WriteError) is returned in case some sections could not be created.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Seek, SeekFrom};
    /// use bpx::utils::new_byte_buf;
    /// use bpx::variant::NamedTable;
    /// use bpx::variant::package::{PackageBuilder, PackageDecoder};
    ///
    /// let mut bpxp = PackageBuilder::new().build(new_byte_buf(0)).unwrap();
    /// bpxp.pack_object("TestObject", "This is a test 你好".as_bytes());
    /// bpxp.save().unwrap();
    /// //Reset our bytebuf pointer to start
    /// let mut bytebuf = bpxp.into_inner().into_inner();
    /// bytebuf.seek(SeekFrom::Start(0)).unwrap();
    /// //Attempt decoding our in-memory BPXP
    /// let mut bpxp = PackageDecoder::new(bytebuf).unwrap();
    /// let table = bpxp.read_object_table().unwrap();
    /// assert_eq!(table.get_all().len(), 1);
    /// let object = table.get_all()[0];
    /// assert_eq!(bpxp.get_object_name(&object).unwrap(), "TestObject");
    /// let mut data = Vec::new();
    /// bpxp.unpack_object(&object, &mut data);
    /// let s = std::str::from_utf8(&data).unwrap();
    /// assert_eq!(s, "This is a test 你好")
    /// ```
    pub fn build<TBackend: IoBackend>(
        self,
        backend: TBackend
    ) -> Result<PackageEncoder<TBackend>, WriteError>
    {
        let mut encoder = Encoder::new(backend)?;
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
            .with_type(b'P')
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
        let strings = encoder.create_section(strings_header)?.clone();
        let object_table = encoder.create_section(object_table_header)?.clone();
        if let Some(obj) = self.metadata {
            let metadata_header = SectionHeaderBuilder::new()
                .with_checksum(Checksum::Weak)
                .with_compression(CompressionMethod::Zlib)
                .with_type(SECTION_TYPE_SD)
                .build();
            let metadata = encoder.create_section(metadata_header)?.clone();
            let mut data = metadata.open()?;
            //TODO: Check
            obj.write(data.as_mut())?;
        }
        return Ok(PackageEncoder {
            strings: StringSection::new(strings),
            encoder,
            last_data_section: None,
            object_table
        });
    }
}

/// Represents a BPX Package encoder.
pub struct PackageEncoder<TBackend: IoBackend>
{
    strings: StringSection,
    last_data_section: Option<Rc<AutoSection>>,
    object_table: Rc<AutoSection>,
    encoder: Encoder<TBackend>
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

impl<TBackend: IoBackend> PackageEncoder<TBackend>
{
    fn write_object<TRead: Read>(
        &mut self,
        source: &mut TRead,
        data_id: &Rc<AutoSection>
    ) -> Result<(usize, bool), crate::error::WriteError>
    {
        //TODO: Fix
        let mut data = data_id.open()?;
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
        return Ok((count, false));
    }

    /// Stores an object in this BPXP with the given name.
    ///
    /// **This function prints some information to standard output as a way
    /// to debug data compression issues unless the `debug-log` feature
    /// is disabled.**
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the object.
    /// * `source`: the source object data as a [Read](std::io::Read).
    ///
    /// returns: Result<(), WriteError>
    ///
    /// # Errors
    ///
    /// A [WriteError](crate::variant::package::error::WriteError) is returned if the object could not be written.
    pub fn pack_object<TRead: Read>(
        &mut self,
        name: &str,
        mut source: TRead
    ) -> Result<(), WriteError>
    {
        let mut object_size = 0;
        let useless = &mut self.encoder;
        let mut data_section = self
            .last_data_section
            .get_or_insert_with_err(|| -> Result<Rc<AutoSection>, crate::error::WriteError> {
                //Here Rust type inference is even more broken! Cloning needs to be done twice!!!!
                let fuckyourust = useless.create_section(create_data_section_header())?;
                return Ok(fuckyourust.clone());
            })?
            .clone();
        let start = self.encoder.get_section_index(data_section.handle());
        let offset = data_section.size() as u32;

        loop {
            let (count, need_section) = self.write_object(&mut source, &data_section)?;
            object_size += count;
            if need_section {
                data_section = self
                    .encoder
                    .create_section(create_data_section_header())?
                    .clone();
            } else {
                break;
            }
        }
        {
            // Fill and write the object header
            let buf = ObjectHeader {
                size: object_size as u64,
                name: self.strings.put(name)?,
                start,
                offset
            }
            .to_bytes();
            // Write the object header
            let mut object_table = self.object_table.open()?;
            object_table.write_all(&buf)?;
        }
        //TODO: Fix
        if data_section.size() > MAX_DATA_SECTION_SIZE {
            self.last_data_section = None;
        } else {
            self.last_data_section = Some(data_section);
        }
        return Ok(());
    }

    /// Saves this BPXP.
    ///
    /// **This function prints some information to standard output as a way
    /// to debug data compression issues unless the `debug-log` feature
    /// is disabled.**
    ///
    /// # Errors
    ///
    /// A [WriteError](crate::error::WriteError) is returned if the encoder failed to save.
    pub fn save(&mut self) -> Result<(), crate::error::WriteError>
    {
        return self.encoder.save();
    }

    /// Consumes this BPXP encoder and returns the inner BPX encoder.
    pub fn into_inner(self) -> Encoder<TBackend>
    {
        return self.encoder;
    }
}
