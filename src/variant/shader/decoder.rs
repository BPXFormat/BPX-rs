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

use std::io::SeekFrom;
use byteorder::{ByteOrder, LittleEndian};
use crate::decoder::{Decoder, IoBackend};
use crate::{Interface, Result, SectionHandle};
use crate::error::Error;
use crate::header::SECTION_TYPE_STRING;
use crate::sd::Object;
use crate::strings::StringSection;
use crate::utils::OptionExtension;
use crate::variant::shader::{SECTION_TYPE_EXTENDED_DATA, SECTION_TYPE_SYMBOL_TABLE, SUPPORTED_VERSION, Target, Type};
use crate::variant::shader::symbol::{FLAG_EXTENDED_DATA, Symbol, SymbolTable, SymbolType};

fn get_target_type_from_code(acode: u8, tcode: u8) -> Result<(Target, Type)>
{
    let target;
    let btype;

    match acode {
        0x1 => target = Target::DX11,
        0x2 => target = Target::DX12,
        0x3 => target = Target::GL33,
        0x4 => target = Target::GL40,
        0x5 => target = Target::VK10,
        0x6 => target = Target::MT,
        0xFF => target = Target::Any,
        _ => return Err(Error::Corruption(String::from("Target code does not exist")))
    }
    if tcode == 'A' as u8 { //Rust refuses to parse match properly so use if/else-if blocks
        btype = Type::Assembly;
    } else if tcode == 'P' as u8 {
        btype = Type::Pipeline;
    } else {
        return Err(Error::Corruption(String::from("Type code does not exist")));
    }
    return Ok((target, btype));
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

pub struct ShaderDecoder<TBackend: IoBackend>
{
    decoder: Decoder<TBackend>,
    assembly_hash: u64,
    num_symbols: u16,
    target: Target,
    btype: Type,
    symbol_table: SectionHandle,
    strings: StringSection,
    extended_data: Option<SectionHandle>
}

impl<TBackend: IoBackend> ShaderDecoder<TBackend>
{
    /// Creates a new ShaderDecoder by reading from a BPX decoder.
    ///
    /// # Arguments
    ///
    /// * `backend`: the [IoBackend](crate::decoder::IoBackend) to use.
    ///
    /// returns: Result<ShaderDecoder<TBackend>, Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned if some sections/headers could not be loaded.
    pub fn new(backend: TBackend) -> Result<ShaderDecoder<TBackend>>
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
                "This version of the BPX SDK only supports BPXS version {}, you are trying to decode version {} BPXS",
                SUPPORTED_VERSION,
                decoder.get_main_header().version
            )));
        }
        let hash = LittleEndian::read_u64(&decoder.get_main_header().type_ext[0..8]);
        let num_symbols = LittleEndian::read_u16(&decoder.get_main_header().type_ext[8..10]);
        let (target, btype) = get_target_type_from_code(decoder.get_main_header().type_ext[10], decoder.get_main_header().type_ext[11])?;
        let strings = match decoder.find_section_by_type(SECTION_TYPE_STRING) {
            Some(v) => v,
            None => return Err(Error::Corruption(String::from("Unable to locate strings section")))
        };
        let symbol_table = match decoder.find_section_by_type(SECTION_TYPE_SYMBOL_TABLE) {
            Some(v) => v,
            None => return Err(Error::Corruption(String::from("Unable to locate BPXS symbol table")))
        };
        return Ok(ShaderDecoder {
            decoder,
            assembly_hash: hash,
            num_symbols,
            target,
            btype,
            symbol_table,
            strings: StringSection::new(strings),
            extended_data: None
        });
    }

    /// Returns the shader package type (Assembly or Pipeline).
    pub fn get_type(&self) -> Type
    {
        return self.btype;
    }

    /// Returns the shader target rendering API.
    pub fn get_target(&self) -> Target
    {
        return self.target;
    }

    /// Returns the number of symbols contained in that BPX.
    pub fn get_symbol_count(&self) -> u16
    {
        return self.num_symbols;
    }

    /// Returns the hash of the shader assembly this pipeline is linked to.
    pub fn get_assembly_hash(&self) -> u64
    {
        return self.assembly_hash;
    }

    /// Gets the name of a symbol; loads the string if its not yet loaded.
    ///
    /// # Arguments
    ///
    /// * `obj`: the symbol to load the actual name for.
    ///
    /// returns: Result<&str, Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned if the name could not be read.
    pub fn get_object_name(&mut self, sym: &Symbol) -> Result<&str>
    {
        return self.strings.get(&mut self.decoder, sym.name);
    }

    /// Reads the symbol table of this BPXS.
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned in case of corruption or system error.
    pub fn read_symbol_table(&mut self) -> Result<SymbolTable>
    {
        let mut v = Vec::new();
        let count = self.decoder.get_section_header(self.symbol_table).size / 20;
        let symbol_table = self.decoder.open_section(self.symbol_table)?;

        for _ in 0..count {
            let mut buf: [u8; 12] = [0; 12];
            if symbol_table.read(&mut buf)? != 12 {
                return Err(Error::Truncation("read symbol table"));
            }
            let name = LittleEndian::read_u32(&buf[0..4]);
            let extended_data = LittleEndian::read_u32(&buf[4..8]);
            let flags = LittleEndian::read_u16(&buf[8..10]);
            let stype = get_symbol_type_from_code(buf[10])?;
            let register = buf[11];

            v.push(Symbol {
                name,
                extended_data,
                flags,
                stype,
                register
            })
        }
        return Ok(SymbolTable::new(v));
    }

    /// Reads the extended data object of a symbol.
    ///
    /// # Arguments
    ///
    /// * `sym`: the symbol to read extended data from.
    ///
    /// returns: Result<Object, Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned in case of corruption or system error.
    ///
    /// # Panics
    ///
    /// Panics if the symbol extended data is undefined.
    pub fn read_extended_data(&mut self, sym: &Symbol) -> Result<Object>
    {
        if sym.flags & FLAG_EXTENDED_DATA == 0 {
            panic!("The symbol extended data is undefined.");
        }
        let useless = &self.decoder;
        let handle = *self.extended_data.get_or_insert_with_err(|| {
            return match useless.find_section_by_type(SECTION_TYPE_EXTENDED_DATA) {
                Some(v) => Ok(v),
                None => Err(Error::Corruption(String::from("Unable to locate ExtendedData section")))
            };
        })?;
        let mut data = self.decoder.open_section(handle)?;
        data.seek(SeekFrom::Start(sym.extended_data as _))?;
        let obj = Object::read(&mut data)?;
        return Ok(obj);
    }
}

