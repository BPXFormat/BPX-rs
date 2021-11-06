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

use std::rc::Rc;

use byteorder::{ByteOrder, LittleEndian};

use crate::{
    builder::{Checksum, CompressionMethod, MainHeaderBuilder, SectionHeaderBuilder},
    encoder::{Encoder, IoBackend},
    header::{Struct, SECTION_TYPE_STRING},
    sd::Object,
    section::AutoSection,
    strings::StringSection,
    utils::OptionExtension,
    variant::shader::{
        error::WriteError,
        symbol::{Symbol, SymbolType},
        Shader,
        Stage,
        Target,
        Type,
        SECTION_TYPE_EXTENDED_DATA,
        SECTION_TYPE_SHADER,
        SECTION_TYPE_SYMBOL_TABLE,
        SUPPORTED_VERSION
    },
    Interface
};

/// Utility to easily generate a [ShaderPackEncoder](crate::variant::shader::ShaderPackEncoder).
pub struct ShaderPackBuilder
{
    assembly_hash: u64,
    target: Target,
    btype: Type
}

impl Default for ShaderPackBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl ShaderPackBuilder
{
    /// Creates a new BPX Shader Package builder.
    pub fn new() -> ShaderPackBuilder
    {
        ShaderPackBuilder {
            assembly_hash: 0,
            target: Target::Any,
            btype: Type::Pipeline
        }
    }

    /// Defines the shader assembly this package is linked against.
    ///
    /// *By default, no shader assembly is linked and the hash is 0.*
    ///
    /// # Arguments
    ///
    /// * `hash`: the shader assembly hash.
    ///
    /// returns: ShaderPackBuilder
    pub fn with_assembly(mut self, hash: u64) -> Self
    {
        self.assembly_hash = hash;
        self
    }

    /// Defines the target of this shader package.
    ///
    /// *By default, the target is Any.*
    ///
    /// # Arguments
    ///
    /// * `target`: the shader target.
    ///
    /// returns: ShaderPackBuilder
    pub fn with_target(mut self, target: Target) -> Self
    {
        self.target = target;
        self
    }

    /// Defines the shader package type.
    ///
    /// *By default, the type is Pipeline.*
    ///
    /// # Arguments
    ///
    /// * `btype`: the shader package type (pipeline/program or assembly).
    ///
    /// returns: ShaderPackBuilder
    pub fn with_type(mut self, btype: Type) -> Self
    {
        self.btype = btype;
        self
    }

    /// Builds the corresponding [ShaderPackEncoder](crate::variant::shader::ShaderPackEncoder).
    ///
    /// # Arguments
    ///
    /// * `backend`: the [IoBackend](crate::encoder::IoBackend) to use.
    ///
    /// returns: Result<ShaderPackEncoder<TBackend>, Error>
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Seek, SeekFrom};
    /// use bpx::utils::new_byte_buf;
    /// use bpx::variant::package::{PackageBuilder, PackageDecoder};
    /// use bpx::variant::shader::{Shader, ShaderPackBuilder, ShaderPackDecoder, Stage};
    /// use bpx::variant::shader::symbol::SymbolType;
    ///
    /// let mut bpxs = ShaderPackBuilder::new().build(new_byte_buf(0)).unwrap();
    /// bpxs.write_symbol("test", SymbolType::Constant, 0, 0xFF, None).unwrap();
    /// bpxs.write_shader(Shader {
    ///     stage: Stage::Pixel,
    ///     data: Vec::new()
    /// }).unwrap();
    /// bpxs.save();
    /// //Reset our bytebuf pointer to start
    /// let mut bytebuf = bpxs.into_inner().into_inner();
    /// bytebuf.seek(SeekFrom::Start(0)).unwrap();
    /// //Attempt decoding our in-memory BPXP
    /// let mut bpxs = ShaderPackDecoder::new(bytebuf).unwrap();
    /// let (items, mut names) = bpxs.read_symbol_table().unwrap();
    /// assert_eq!(items.len(), 1);
    /// assert!(!items.is_empty());
    /// let sym = items[0];
    /// assert_eq!(bpxs.get_symbol_count(), 1);
    /// assert_eq!(names.load(&sym).unwrap(), "test");
    /// let shader = bpxs.load_shader(bpxs.list_shaders()[0]).unwrap();
    /// assert_eq!(shader.stage, Stage::Pixel);
    /// assert_eq!(shader.data.len(), 0);
    /// ```
    pub fn build<TBackend: IoBackend>(
        self,
        backend: TBackend
    ) -> Result<ShaderPackEncoder<TBackend>, WriteError>
    {
        let mut encoder = Encoder::new(backend)?;
        let mut type_ext: [u8; 16] = [0; 16];
        match self.target {
            Target::DX11 => type_ext[10] = 0x1,
            Target::DX12 => type_ext[10] = 0x2,
            Target::GL33 => type_ext[10] = 0x3,
            Target::GL40 => type_ext[10] = 0x4,
            Target::VK10 => type_ext[10] = 0x5,
            Target::MT => type_ext[10] = 0x6,
            Target::Any => type_ext[10] = 0xFF
        };
        match self.btype {
            Type::Assembly => type_ext[11] = b'A',
            Type::Pipeline => type_ext[11] = b'P'
        };
        LittleEndian::write_u64(&mut type_ext[0..8], self.assembly_hash);
        let header = MainHeaderBuilder::new()
            .with_type(b'P')
            .with_type_ext(type_ext)
            .with_version(SUPPORTED_VERSION)
            .build();
        encoder.set_main_header(header);
        let symbol_table_header = SectionHeaderBuilder::new()
            .with_checksum(Checksum::Weak)
            .with_compression(CompressionMethod::Zlib)
            .with_type(SECTION_TYPE_SYMBOL_TABLE)
            .build();
        let strings_header = SectionHeaderBuilder::new()
            .with_checksum(Checksum::Weak)
            .with_compression(CompressionMethod::Zlib)
            .with_type(SECTION_TYPE_STRING)
            .build();
        let strings = encoder.create_section(strings_header)?.clone();
        let symbol_table = encoder.create_section(symbol_table_header)?.clone();
        Ok(ShaderPackEncoder {
            encoder,
            strings: StringSection::new(strings),
            extended_data: None,
            symbol_table,
            num_symbols: 0
        })
    }
}

/// Represents a BPX Shader Package encoder.
pub struct ShaderPackEncoder<TBackend: IoBackend>
{
    encoder: Encoder<TBackend>,
    strings: StringSection,
    extended_data: Option<Rc<AutoSection>>,
    symbol_table: Rc<AutoSection>,
    num_symbols: u16
}

impl<TBackend: IoBackend> ShaderPackEncoder<TBackend>
{
    fn write_extended_data(&mut self, extended_data: Option<Object>) -> Result<u32, WriteError>
    {
        if let Some(obj) = extended_data {
            let useless = &mut self.encoder;
            let handle = self.extended_data.get_or_insert_with_err(
                || -> Result<Rc<AutoSection>, crate::error::WriteError> {
                    let header = SectionHeaderBuilder::new()
                        .with_type(SECTION_TYPE_EXTENDED_DATA)
                        .with_checksum(Checksum::Crc32)
                        .with_compression(CompressionMethod::Zlib)
                        .build();
                    let fuckyourust = useless.create_section(header)?;
                    Ok(fuckyourust.clone())
                }
            )?;
            let mut section = handle.open()?;
            let offset = section.size();
            //TODO: Check
            obj.write(section.as_mut())?;
            return Ok(offset as u32);
        }
        Ok(0xFFFFFF)
    }

    fn patch_extended_data(&mut self)
    {
        let mut header = *self.encoder.get_main_header();
        LittleEndian::write_u16(&mut header.type_ext[8..10], self.num_symbols);
        self.encoder.set_main_header(header);
    }

    /// Writes a symbol into this BPXS.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the symbols.
    /// * `stype`: the [SymbolType](crate::variant::shader::symbol::SymbolType).
    /// * `flags`: the symbol flags (see the FLAG_ constants in the [symbol](crate::variant::shader::symbol) module).
    /// * `register`: the register number of this symbol.
    /// * `extended_data`: an optional BPXSD object to write as extended symbol data.
    ///
    /// returns: Result<(), Error>
    ///
    /// # Errors
    ///
    /// A [WriteError](crate::variant::shader::error::WriteError) is returned if the symbol could not be written.
    pub fn write_symbol<T: AsRef<str>>(
        &mut self,
        name: T,
        stype: SymbolType,
        flags: u16,
        register: u8,
        extended_data: Option<Object>
    ) -> Result<(), WriteError>
    {
        let address = self.strings.put(name.as_ref())?;
        let extended_data = self.write_extended_data(extended_data)?;
        let buf = Symbol {
            name: address,
            extended_data,
            flags,
            stype,
            register
        }
        .to_bytes();
        {
            let mut data = self.symbol_table.open()?;
            data.write_all(&buf)?;
        } //Rust borrow checker is so stupid not able to understand that data is not used after this line
          //So we have to add another scope to workarround that defect
        self.num_symbols += 1;
        self.patch_extended_data();
        Ok(())
    }

    /// Writes a shader into this BPXS.
    ///
    /// # Arguments
    ///
    /// * `shader`: the [Shader](crate::variant::shader::Shader) to write.
    ///
    /// returns: Result<(), Error>
    ///
    /// # Errors
    ///
    /// A [WriteError](crate::variant::shader::error::WriteError) is returned if the shader could not be written.
    pub fn write_shader(&mut self, shader: Shader) -> Result<(), WriteError>
    {
        let section = self.encoder.create_section(
            SectionHeaderBuilder::new()
                .with_type(SECTION_TYPE_SHADER)
                .with_checksum(Checksum::Crc32)
                .with_compression(CompressionMethod::Xz)
                .with_size(shader.data.len() as u32 + 1)
                .build()
        )?;
        let mut data = section.open()?;
        let mut buf = shader.data;
        match shader.stage {
            Stage::Vertex => buf.insert(0, 0x0),
            Stage::Hull => buf.insert(0, 0x1),
            Stage::Domain => buf.insert(0, 0x2),
            Stage::Geometry => buf.insert(0, 0x3),
            Stage::Pixel => buf.insert(0, 0x4)
        };
        data.write_all(&buf)?;
        Ok(())
    }

    /// Saves this BPXS.
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
        self.encoder.save()
    }

    /// Consumes this BPXS encoder and returns the inner BPX encoder.
    pub fn into_inner(self) -> Encoder<TBackend>
    {
        self.encoder
    }
}
