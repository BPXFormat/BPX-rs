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

//! Contains utilities to work with the symbol table section.

use std::{collections::HashMap, io::Read};

use byteorder::{ByteOrder, LittleEndian};

use crate::{decoder::IoBackend, variant::shader::ShaderPackDecoder};
use crate::variant::{BuildNamedTable, NamedTable};
use crate::variant::shader::error::{EosContext, ReadError};

/// Indicates this symbol is used on the vertex stage.
pub const FLAG_VERTEX_STAGE: u16 = 0x1;

/// Indicates this symbol is used on the hull stage.
pub const FLAG_HULL_STAGE: u16 = 0x2;

/// Indicates this symbol is used on the domain stage.
pub const FLAG_DOMAIN_STAGE: u16 = 0x4;

/// Indicates this symbol is used on the geometry stage.
pub const FLAG_GEOMETRY_STAGE: u16 = 0x8;

/// Indicates this symbol is used on the pixel stage.
pub const FLAG_PIXEL_STAGE: u16 = 0x10;

/// Indicates this symbol is part of an assembly.
pub const FLAG_ASSEMBLY: u16 = 0x20;

/// Indicates this symbol is not defined in this package.
pub const FLAG_EXTERNAL: u16 = 0x40;

/// Indicates this symbol is defined in this package.
pub const FLAG_INTERNAL: u16 = 0x80;

/// Indicates this symbol has extended data.
pub const FLAG_EXTENDED_DATA: u16 = 0x100;

/// Indicates this symbol has a register number.
pub const FLAG_REGISTER: u16 = 0x200;

/// Size in bytes of a symbol structure.
pub const SYMBOL_STRUCTURE_SIZE: usize = 12;

/// The type of a symbol.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SymbolType
{
    /// A texture symbol.
    Texture,

    /// A sampler symbol.
    Sampler,

    /// A constant buffer symbol.
    ConstantBuffer,

    /// A high performance constant symbol (represented as push constants in vulkan).
    Constant,

    /// A vertex format symbol.
    VertexFormat,

    /// A pipeline symbol.
    Pipeline
}

fn get_symbol_type_from_code(scode: u8) -> Result<SymbolType, ReadError>
{
    return match scode {
        0x0 => Ok(SymbolType::Texture),
        0x1 => Ok(SymbolType::Sampler),
        0x2 => Ok(SymbolType::ConstantBuffer),
        0x3 => Ok(SymbolType::Constant),
        0x4 => Ok(SymbolType::VertexFormat),
        0x5 => Ok(SymbolType::Pipeline),
        _ => Err(ReadError::InvalidSymbolTypeCode(scode))
    };
}

/// Represents the structure of a symbol.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Symbol
{
    /// The pointer to the name of the symbol.
    pub name: u32,

    /// The pointer to the BPXSD object attached to this symbol.
    pub extended_data: u32,

    /// The symbol flags (see the FLAG_ constants in the [symbol](crate::variant::shader::symbol) module).
    pub flags: u16,

    /// The type of symbol.
    pub stype: SymbolType,

    /// The register number for this symbol.
    pub register: u8
}

impl Symbol
{
    /// Reads a symbol structure from a [Read](std::io::Read)
    ///
    /// # Arguments
    ///
    /// * `reader`: the [Read](std::io::Read) to read from.
    ///
    /// returns: Result<Symbol, Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned in case of data truncation.
    pub fn read<TReader: Read>(reader: &mut TReader) -> Result<Symbol, ReadError>
    {
        let mut buf: [u8; SYMBOL_STRUCTURE_SIZE] = [0; SYMBOL_STRUCTURE_SIZE];
        if reader.read(&mut buf)? != 12 {
            return Err(ReadError::Eos(EosContext::SymbolTable));
        }
        let name = LittleEndian::read_u32(&buf[0..4]);
        let extended_data = LittleEndian::read_u32(&buf[4..8]);
        let flags = LittleEndian::read_u16(&buf[8..10]);
        let stype = get_symbol_type_from_code(buf[10])?;
        let register = buf[11];
        return Ok(Symbol {
            name,
            extended_data,
            flags,
            stype,
            register
        });
    }

    /// Converts this symbol structure to a byte array.
    pub fn to_bytes(&self) -> [u8; SYMBOL_STRUCTURE_SIZE]
    {
        let mut buf = [0; SYMBOL_STRUCTURE_SIZE];
        LittleEndian::write_u32(&mut buf[0..4], self.name);
        LittleEndian::write_u32(&mut buf[4..8], self.extended_data);
        LittleEndian::write_u16(&mut buf[8..10], self.flags);
        match self.stype {
            SymbolType::Texture => buf[10] = 0x0,
            SymbolType::Sampler => buf[10] = 0x1,
            SymbolType::ConstantBuffer => buf[10] = 0x2,
            SymbolType::Constant => buf[10] = 0x3,
            SymbolType::VertexFormat => buf[10] = 0x4,
            SymbolType::Pipeline => buf[10] = 0x5
        };
        buf[11] = self.register;
        return buf;
    }
}

/// Helper class to query a symbol table.
pub struct SymbolTable
{
    list: Vec<Symbol>,
    map: Option<HashMap<String, Symbol>>
}

impl NamedTable for SymbolTable
{
    type Inner = Symbol;

    fn new(list: Vec<Self::Inner>) -> Self
    {
        return SymbolTable {
            list,
            map: None
        };
    }

    fn lookup(&self, name: &str) -> Option<&Self::Inner>
    {
        if let Some(map) = &self.map {
            return map.get(name);
        } else {
            panic!("Lookup table has not yet been initialized, please call build_lookup_table");
        }
    }

    fn get_all(&self) -> &[Self::Inner]
    {
        return &self.list;
    }
}

impl<TBackend: IoBackend> BuildNamedTable<ShaderPackDecoder<TBackend>> for SymbolTable
{
    fn build_lookup_table(&mut self, package: &mut ShaderPackDecoder<TBackend>) -> Result<(), crate::strings::ReadError>
    {
        let mut map = HashMap::new();
        for v in &self.list {
            let name = String::from(package.get_symbol_name(v)?);
            map.insert(name, *v);
        }
        self.map = Some(map);
        return Ok(());
    }
}
