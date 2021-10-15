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

use std::io::Write;
use byteorder::{ByteOrder, LittleEndian};
use crate::encoder::{Encoder, IoBackend};
use crate::variant::shader::{SECTION_TYPE_EXTENDED_DATA, SECTION_TYPE_SHADER, SECTION_TYPE_SYMBOL_TABLE, Shader, Stage, SUPPORTED_VERSION, Target, Type};
use crate::{Interface, Result, SectionHandle};
use crate::builder::{Checksum, CompressionMethod, MainHeaderBuilder, SectionHeaderBuilder};
use crate::header::SECTION_TYPE_STRING;
use crate::sd::Object;
use crate::strings::StringSection;
use crate::utils::OptionExtension;
use crate::variant::shader::symbol::{Symbol, SymbolType};

pub struct ShaderPackBuilder
{
    assembly_hash: u64,
    target: Target,
    btype: Type
}

impl ShaderPackBuilder
{
    pub fn new() -> ShaderPackBuilder
    {
        return ShaderPackBuilder {
            assembly_hash: 0,
            target: Target::Any,
            btype: Type::Pipeline
        };
    }

    pub fn with_assembly(mut self, hash: u64) -> Self
    {
        self.assembly_hash = hash;
        return self;
    }

    pub fn with_target(mut self, target: Target) -> Self
    {
        self.target = target;
        return self;
    }

    pub fn with_type(mut self, btype: Type) -> Self
    {
        self.btype = btype;
        return self;
    }

    pub fn build<TBackend: IoBackend>(mut self, backend: TBackend) -> Result<ShaderPackEncoder<TBackend>>
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
            Type::Assembly => type_ext[11] = 'A' as u8,
            Type::Pipeline => type_ext[11] = 'P' as u8
        };
        LittleEndian::write_u64(&mut type_ext[0..8], self.assembly_hash);
        let header = MainHeaderBuilder::new()
            .with_type('P' as u8)
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
        let strings = encoder.create_section(strings_header)?;
        let symbol_table = encoder.create_section(symbol_table_header)?;
        return Ok(ShaderPackEncoder {
            encoder,
            strings: StringSection::new(strings),
            extended_data: None,
            symbol_table,
            num_symbols: 0
        });
    }
}

pub struct ShaderPackEncoder<TBackend: IoBackend>
{
    encoder: Encoder<TBackend>,
    strings: StringSection,
    extended_data: Option<SectionHandle>,
    symbol_table: SectionHandle,
    num_symbols: u16
}

impl<TBackend: IoBackend> ShaderPackEncoder<TBackend>
{
    fn write_extended_data(&mut self, extended_data: Option<Object>) -> Result<u32>
    {
        if let Some(obj) = extended_data {
            let useless = &mut self.encoder;
            let handle = *self.extended_data.get_or_insert_with_err(|| {
                let header = SectionHeaderBuilder::new()
                    .with_type(SECTION_TYPE_EXTENDED_DATA)
                    .with_checksum(Checksum::Crc32)
                    .with_compression(CompressionMethod::Zlib)
                    .build();
                return useless.create_section(header);
            })?;
            let mut section = self.encoder.open_section(handle)?;
            let offset = section.size();
            obj.write(&mut section)?;
            return Ok(offset as u32);
        }
        return Ok(0xFFFFFF);
    }

    fn patch_extended_data(&mut self)
    {
        let mut header = *self.encoder.get_main_header();
        LittleEndian::write_u16(&mut header.type_ext[8..10], self.num_symbols);
        self.encoder.set_main_header(header);
    }

    pub fn write_symbol<T: AsRef<str>>(&mut self, name: T, stype: SymbolType, flags: u16, register: u8, extended_data: Option<Object>) -> Result<()>
    {
        let address = self.strings.put(&mut self.encoder, name.as_ref())?;
        let extended_data = self.write_extended_data(extended_data)?;
        let sym = Symbol {
            name: address,
            extended_data,
            flags,
            stype,
            register
        };
        let data = self.encoder.open_section(self.symbol_table)?;
        data.write(&sym.to_bytes())?;
        self.num_symbols += 1;
        self.patch_extended_data();
        return Ok(());
    }

    pub fn write_shader(&mut self, shader: Shader) -> Result<()>
    {
        let section = self.encoder.create_section(
            SectionHeaderBuilder::new()
                .with_type(SECTION_TYPE_SHADER)
                .with_checksum(Checksum::Crc32)
                .with_compression(CompressionMethod::Xz)
                .with_size(shader.data.len() as u32 + 1)
                .build()
        )?;
        let data = self.encoder.open_section(section)?;
        let mut buf = shader.data;
        match shader.stage {
            Stage::Vertex => buf.insert(0, 0x0),
            Stage::Hull => buf.insert(0, 0x1),
            Stage::Domain => buf.insert(0, 0x2),
            Stage::Geometry => buf.insert(0, 0x3),
            Stage::Pixel => buf.insert(0, 0x4)
        };
        data.write(&buf)?;
        return Ok(());
    }

    pub fn save(&mut self) -> Result<()>
    {
        return self.encoder.save();
    }
}
