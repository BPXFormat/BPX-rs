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
use std::ops::DerefMut;
use std::rc::Rc;

use byteorder::{ByteOrder, LittleEndian};

use crate::{
    decoder::{Decoder, IoBackend},
    header::SECTION_TYPE_STRING,
    sd::Object,
    strings::StringSection,
    utils::OptionExtension,
    variant::shader::{
        symbol::{Symbol, SymbolTable, FLAG_EXTENDED_DATA},
        Shader,
        Stage,
        Target,
        Type,
        SECTION_TYPE_EXTENDED_DATA,
        SECTION_TYPE_SHADER,
        SECTION_TYPE_SYMBOL_TABLE,
        SUPPORTED_VERSION
    },
    Interface,
    SectionHandle
};
use crate::header::Struct;
use crate::section::{AutoSection};
use crate::variant::NamedTable;
use crate::variant::shader::error::{EosContext, ReadError, Section};
use crate::variant::shader::symbol::SIZE_SYMBOL_STRUCTURE;

fn get_target_type_from_code(acode: u8, tcode: u8) -> Result<(Target, Type), ReadError>
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
        _ => return Err(ReadError::InvalidTargetCode(acode))
    }
    if tcode == 'A' as u8 {
        //Rust refuses to parse match properly so use if/else-if blocks
        btype = Type::Assembly;
    } else if tcode == 'P' as u8 {
        btype = Type::Pipeline;
    } else {
        return Err(ReadError::InvalidTypeCode(tcode));
    }
    return Ok((target, btype));
}

fn get_stage_from_code(code: u8) -> Result<Stage, ReadError>
{
    return match code {
        0x0 => Ok(Stage::Vertex),
        0x1 => Ok(Stage::Hull),
        0x2 => Ok(Stage::Domain),
        0x3 => Ok(Stage::Geometry),
        0x4 => Ok(Stage::Pixel),
        _ => Err(ReadError::InvalidStageCode(code))
    };
}

/// Represents a BPX Shader Package decoder.
pub struct ShaderPackDecoder<TBackend: IoBackend>
{
    decoder: Decoder<TBackend>,
    assembly_hash: u64,
    num_symbols: u16,
    target: Target,
    btype: Type,
    symbol_table: Rc<AutoSection>,
    strings: StringSection,
    extended_data: Option<Rc<AutoSection>>
}

impl<TBackend: IoBackend> ShaderPackDecoder<TBackend>
{
    /// Creates a new ShaderPackDecoder by reading from a BPX decoder.
    ///
    /// # Arguments
    ///
    /// * `backend`: the [IoBackend](crate::decoder::IoBackend) to use.
    ///
    /// returns: Result<ShaderPackDecoder<TBackend>, Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned if some sections/headers could not be loaded.
    pub fn new(backend: TBackend) -> Result<ShaderPackDecoder<TBackend>, ReadError>
    {
        let mut decoder = Decoder::new(backend)?;
        if decoder.get_main_header().btype != 'P' as u8 {
            return Err(ReadError::BadType(decoder.get_main_header().btype));
        }
        if decoder.get_main_header().version != SUPPORTED_VERSION {
            return Err(ReadError::BadVersion(decoder.get_main_header().version));
        }
        let hash = LittleEndian::read_u64(&decoder.get_main_header().type_ext[0..8]);
        let num_symbols = LittleEndian::read_u16(&decoder.get_main_header().type_ext[8..10]);
        let (target, btype) = get_target_type_from_code(
            decoder.get_main_header().type_ext[10],
            decoder.get_main_header().type_ext[11]
        )?;
        let strings = match decoder.find_section_by_type(SECTION_TYPE_STRING) {
            Some(v) => v,
            None => return Err(ReadError::MissingSection(Section::Strings))
        };
        let symbol_table = match decoder.find_section_by_type(SECTION_TYPE_SYMBOL_TABLE) {
            Some(v) => v,
            None => return Err(ReadError::MissingSection(Section::SymbolTable))
        };
        return Ok(ShaderPackDecoder {
            assembly_hash: hash,
            num_symbols,
            target,
            btype,
            symbol_table: decoder.load_section(symbol_table)?.clone(),
            strings: StringSection::new(decoder.load_section(strings)?.clone()),
            extended_data: None,
            decoder
        });
    }

    /// Lists all shaders contained in this shader package.
    pub fn list_shaders(&self) -> Vec<SectionHandle>
    {
        return self.decoder.find_all_sections_of_type(SECTION_TYPE_SHADER);
    }

    /// Loads a shader into memory.
    ///
    /// # Arguments
    ///
    /// * `handle`: a handle to the shader section.
    ///
    /// returns: Result<Shader, Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned if the shader could not be loaded.
    pub fn load_shader(&mut self, handle: SectionHandle) -> Result<Shader, ReadError>
    {
        let header = self.decoder.get_section_header(handle);
        if header.size < 1 {
            //We must at least find a stage byte
            return Err(ReadError::Eos(EosContext::Shader));
        }
        let s = self.decoder.load_section(handle)?;
        let mut section = s.open()?;
        let mut buf = section.load_in_memory()?;
        let stage = get_stage_from_code(buf.remove(0))?;
        return Ok(Shader { stage, data: buf });
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
    /// * `sym`: the symbol to load the actual name for.
    ///
    /// returns: Result<&str, Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned if the name could not be read.
    pub fn get_symbol_name(&mut self, sym: &Symbol) -> Result<&str, crate::strings::ReadError>
    {
        return self.strings.get(sym.name);
    }

    /// Reads the symbol table of this BPXS.
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned in case of corruption or system error.
    pub fn read_symbol_table(&mut self) -> Result<SymbolTable, ReadError>
    {
        use crate::section::Section;
        let mut v = Vec::new();
        let count = self.symbol_table.size() as u32 / SIZE_SYMBOL_STRUCTURE as u32;
        let mut symbol_table = self.symbol_table.open()?;

        if count != self.num_symbols as u32 {
            return Err(ReadError::Eos(EosContext::SymbolTable));
        }
        for _ in 0..count {
            //Type inference in Rust is so buggy! One &mut dyn is not enough you need double &mut dyn now!
            let sym = Symbol::read(&mut symbol_table.deref_mut())?;
            v.push(sym);
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
    pub fn read_extended_data(&mut self, sym: &Symbol) -> Result<Object, ReadError>
    {
        if sym.flags & FLAG_EXTENDED_DATA == 0 {
            panic!("The symbol extended data is undefined.");
        }
        let useless = &mut self.decoder;
        let section = self.extended_data.get_or_insert_with_err(|| {
            return match useless.find_section_by_type(SECTION_TYPE_EXTENDED_DATA) {
                Some(v) => Ok(useless.load_section(v)?.clone()),
                None => Err(ReadError::MissingSection(Section::ExtendedData))
            };
        })?;
        let mut data = section.open()?;
        data.seek(SeekFrom::Start(sym.extended_data as _))?;
        //Type inference in Rust is so buggy! One &mut dyn is not enough you need double &mut dyn now!
        let obj = Object::read(&mut data.deref_mut())?;
        return Ok(obj);
    }

    /// Consumes this BPXS decoder and returns the inner BPX decoder.
    pub fn into_inner(self) -> Decoder<TBackend>
    {
        return self.decoder;
    }
}
