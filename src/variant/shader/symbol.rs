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

use std::collections::HashMap;
use std::io::Read;
use byteorder::{ByteOrder, LittleEndian};
use crate::decoder::IoBackend;
use crate::error::Error;
use crate::Result;
use crate::variant::shader::ShaderPackDecoder;

pub const FLAG_VERTEX_STAGE: u16 = 0x1;
pub const FLAG_HULL_STAGE: u16 = 0x2;
pub const FLAG_DOMAIN_STAGE: u16 = 0x4;
pub const FLAG_GEOMETRY_STAGE: u16 = 0x8;
pub const FLAG_PIXEL_STAGE: u16 = 0x10;
pub const FLAG_ASSEMBLY: u16 = 0x20;
pub const FLAG_EXTERNAL: u16 = 0x40;
pub const FLAG_INTERNAL: u16 = 0x80;
pub const FLAG_EXTENDED_DATA: u16 = 0x100;
pub const FLAG_REGISTER: u16 = 0x200;

pub const SYMBOL_STRUCTURE_SIZE: usize = 12;

#[derive(Copy, Clone)]
pub enum SymbolType
{
    Texture,
    Sampler,
    ConstantBuffer,
    Constant,
    VertexFormat,
    Pipeline
}

fn get_symbol_type_from_code(scode: u8) -> Result<SymbolType>
{
    return match scode {
        0x0 => Ok(SymbolType::Texture),
        0x1 => Ok(SymbolType::Sampler),
        0x2 => Ok(SymbolType::ConstantBuffer),
        0x3 => Ok(SymbolType::Constant),
        0x4 => Ok(SymbolType::VertexFormat),
        0x5 => Ok(SymbolType::Pipeline),
        _ => Err(Error::Corruption(String::from("Symbol type code does not exist")))
    }
}

#[derive(Copy, Clone)]
pub struct Symbol
{
    pub name: u32,
    pub extended_data: u32,
    pub flags: u16,
    pub stype: SymbolType,
    pub register: u8
}

impl Symbol
{
    pub fn read<TReader: Read>(reader: &mut TReader) -> Result<Symbol>
    {
        let mut buf: [u8; SYMBOL_STRUCTURE_SIZE] = [0; SYMBOL_STRUCTURE_SIZE];
        if reader.read(&mut buf)? != 12 {
            return Err(Error::Truncation("read symbol table"));
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

pub struct SymbolTable
{
    list: Vec<Symbol>,
    map: Option<HashMap<String, Symbol>>
}

impl SymbolTable
{
    /// Constructs a new object table from a list of
    /// [ObjectHeader](crate::variant::package::object::ObjectHeader).
    ///
    /// # Arguments
    ///
    /// * `list`: the list of object headers.
    ///
    /// returns: ObjectTable
    pub fn new(list: Vec<Symbol>) -> SymbolTable
    {
        return SymbolTable { list, map: None };
    }

    /// Builds the object map for easy and efficient lookup of objects by name.
    ///
    /// **You must call this function before you can use find_object.**
    ///
    /// # Arguments
    ///
    /// * `package`: the [PackageDecoder](crate::variant::package::PackageDecoder) to load the strings from.
    ///
    /// returns: Result<(), Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned if the strings could
    /// not be loaded.
    pub fn build_lookup_table<TBackend: IoBackend>(&mut self, package: &mut ShaderPackDecoder<TBackend>) -> Result<()>
    {
        let mut map = HashMap::new();
        for v in &self.list {
            let name = String::from(package.get_object_name(v)?);
            map.insert(name, *v);
        }
        self.map = Some(map);
        return Ok(());
    }

    /// Gets all symbols in this BPXS.
    pub fn get_symbols(&self) -> &Vec<Symbol>
    {
        return &self.list;
    }

    /// Finds a symbol by its name.
    /// Returns None if the symbol does not exist.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the symbol to search for.
    ///
    /// returns: Option<&Symbol>
    ///
    /// # Panics
    ///
    /// Panics if the lookup table is not yet built.
    pub fn find_symbol(&self, name: &str) -> Option<&Symbol>
    {
        if let Some(map) = &self.map {
            return map.get(name);
        } else {
            panic!("SymbolTable lookup table has not yet been initialized, please call build_lookup_table");
        }
    }
}
