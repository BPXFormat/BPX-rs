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

use std::io::{Read, Seek, SeekFrom, Write};
use std::ops::Deref;
use std::slice::Iter;
use byteorder::{ByteOrder, LittleEndian};
use crate::core::builder::{Checksum, CompressionMethod, MainHeaderBuilder, SectionHeaderBuilder};
use crate::core::{Container, SectionData};
use crate::core::header::{SECTION_TYPE_STRING, Struct};
use crate::Handle;
use crate::sd::Object;
use crate::shader::{SECTION_TYPE_EXTENDED_DATA, SECTION_TYPE_SHADER, SECTION_TYPE_SYMBOL_TABLE, Settings, Shader, Stage, SUPPORTED_VERSION, Target, Type};
use crate::shader::error::{EosContext, InvalidCodeContext, ReadError, Section, WriteError};
use crate::shader::symbol::{FLAG_EXTENDED_DATA, SIZE_SYMBOL_STRUCTURE, Symbol, SymbolType};
use crate::strings::{load_string_section, StringSection};
use crate::table::ItemTable;
use crate::utils::OptionExtension;

pub struct SymbolRef<'a, T>
{
    extended_data: &'a mut Option<Handle>,
    container: &'a mut Container<T>,
    strings: &'a mut StringSection,
    sym: &'a Symbol
}

impl<'a, T> Deref for SymbolRef<'a, T>
{
    type Target = Symbol;

    fn deref(&self) -> &Self::Target
    {
        self.sym
    }
}

impl<'a, T: Read + Seek> SymbolRef<'a, T>
{
    pub fn load_name(&mut self) -> Result<&str, ReadError>
    {
        load_string_section(self.container, self.strings)?;
        let addr = self.name;
        let str = self.strings.get(self.container, addr)?;
        Ok(str)
    }

    pub fn load_extended_data(&mut self) -> Result<Object, ReadError>
    {
        if self.flags & FLAG_EXTENDED_DATA == 0 {
            panic!("The symbol extended data is undefined.");
        }
        let section = *self.extended_data.get_or_insert_with_err(|| {
            return match self.container.find_section_by_type(SECTION_TYPE_EXTENDED_DATA) {
                Some(v) => Ok(v),
                None => Err(ReadError::MissingSection(Section::ExtendedData))
            };
        })?;
        let mut section = self.container.get_mut(section);
        let data = section.load()?;
        data.seek(SeekFrom::Start(self.sym.extended_data as _))?;
        let obj = Object::read(data)?;
        Ok(obj)
    }
}

pub struct SymbolIter<'a, T>
{
    extended_data: &'a mut Option<Handle>,
    container: &'a mut Container<T>,
    strings: &'a mut StringSection,
    iter: Iter<'a, Symbol>
}

impl<'a, T> Iterator for SymbolIter<'a, T>
{
    type Item = SymbolRef<'a, T>;

    fn next(&mut self) -> Option<Self::Item>
    {
        let sym = self.iter.next()?;
        unsafe {
            let ptr = self.container as *mut Container<T>;
            let ptr1 = self.strings as *mut StringSection;
            let ptr2 = self.extended_data as *mut Option<Handle>;
            Some(SymbolRef {
                extended_data: &mut *ptr2,
                strings: &mut *ptr1,
                container: &mut *ptr,
                sym
            })
        }
    }
}

/// A BPXS (ShaderPack).
///
/// # Examples
///
/// ```
/// use std::io::Seek;
/// use bpx::shader::{Builder, Shader, ShaderPack};
/// use bpx::utils::new_byte_buf;
///
/// let mut bpxs = ShaderPack::create(new_byte_buf(0), Builder::new());
/// bpxs.add_symbol("test", SymbolType::Constant, 0, 0xFF, None).unwrap();
/// bpxs.add_shader(Shader {
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
pub struct ShaderPack<T>
{
    settings: Settings,
    container: Container<T>,
    strings: StringSection,
    symbol_table: Handle,
    symbols: Vec<Symbol>,
    table: Option<ItemTable<Symbol>>,
    extended_data: Option<Handle>,
    num_symbols: u16
}

impl<T> ShaderPack<T>
{
    /// Returns the shader package type (Assembly or Pipeline).
    pub fn get_type(&self) -> Type
    {
        self.settings.btype
    }

    /// Returns the shader target rendering API.
    pub fn get_target(&self) -> Target
    {
        self.settings.target
    }

    /// Returns the number of symbols contained in that BPX.
    pub fn get_symbol_count(&self) -> u16
    {
        self.num_symbols
    }

    /// Returns the hash of the shader assembly this pipeline is linked to.
    pub fn get_assembly_hash(&self) -> u64
    {
        self.settings.assembly_hash
    }

    /// Consumes this ShaderPack and returns the BPX container.
    pub fn into_inner(self) -> Container<T>
    {
        self.container
    }
}

fn get_type_ext(settings: &Settings) -> [u8; 16]
{
    let mut type_ext: [u8; 16] = [0; 16];
    match settings.target {
        Target::DX11 => type_ext[10] = 0x1,
        Target::DX12 => type_ext[10] = 0x2,
        Target::GL33 => type_ext[10] = 0x3,
        Target::GL40 => type_ext[10] = 0x4,
        Target::GL41 => type_ext[10] = 0x5,
        Target::GL42 => type_ext[10] = 0x6,
        Target::GL43 => type_ext[10] = 0x7,
        Target::GL44 => type_ext[10] = 0x8,
        Target::GL45 => type_ext[10] = 0x9,
        Target::GL46 => type_ext[10] = 0xA,
        Target::ES30 => type_ext[10] = 0xB,
        Target::ES31 => type_ext[10] = 0xC,
        Target::ES32 => type_ext[10] = 0xD,
        Target::VK10 => type_ext[10] = 0xE,
        Target::VK11 => type_ext[10] = 0xF,
        Target::VK12 => type_ext[10] = 0x10,
        Target::MT => type_ext[10] = 0x11,
        Target::Any => type_ext[10] = 0xFF
    };
    match settings.btype {
        Type::Assembly => type_ext[11] = b'A',
        Type::Pipeline => type_ext[11] = b'P'
    };
    LittleEndian::write_u64(&mut type_ext[0..8], settings.assembly_hash);
    type_ext
}

impl<T: Write + Seek> ShaderPack<T>
{
    pub fn create<S: Into<Settings>>(backend: T, settings: S) -> ShaderPack<T>
    {
        let settings = settings.into();
        let mut container = Container::create(backend, MainHeaderBuilder::new()
            .with_type(b'P')
            .with_type_ext(get_type_ext(&settings))
            .with_version(SUPPORTED_VERSION));
        let string_section = container.create_section(SectionHeaderBuilder::new()
            .with_checksum(Checksum::Weak)
            .with_compression(CompressionMethod::Zlib)
            .with_type(SECTION_TYPE_STRING));
        let symbol_table = container.create_section(SectionHeaderBuilder::new()
            .with_checksum(Checksum::Weak)
            .with_compression(CompressionMethod::Zlib)
            .with_type(SECTION_TYPE_SYMBOL_TABLE));
        let strings = StringSection::new(string_section);
        ShaderPack {
            container,
            strings,
            settings,
            symbol_table,
            symbols: Vec::new(),
            table: None,
            extended_data: None,
            num_symbols: 0
        }
    }

    fn write_extended_data(&mut self, extended_data: Option<Object>) -> Result<u32, WriteError>
    {
        if let Some(obj) = extended_data {
            let handle = *self.extended_data.get_or_insert_with(
                || self.container.create_section(SectionHeaderBuilder::new()
                                                     .with_type(SECTION_TYPE_EXTENDED_DATA)
                                                     .with_checksum(Checksum::Crc32)
                                                     .with_compression(CompressionMethod::Zlib)));
            let mut section = self.container.get_mut(handle);
            let data = section.open().ok_or(WriteError::SectionNotLoaded)?;
            let offset = data.size();
            obj.write(data)?;
            return Ok(offset as u32);
        }
        Ok(0xFFFFFF)
    }

    fn patch_extended_data(&mut self)
    {
        let mut header = *self.container.get_main_header();
        LittleEndian::write_u16(&mut header.type_ext[8..10], self.num_symbols);
        self.container.set_main_header(header);
    }

    /// Adds a symbol into this BPXS.
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
    pub fn add_symbol<S: AsRef<str>>(
        &mut self,
        name: S,
        stype: SymbolType,
        flags: u16,
        register: u8,
        extended_data: Option<Object>
    ) -> Result<(), WriteError>
    {
        let address = self.strings.put(&mut self.container, name.as_ref())?;
        let extended_data = self.write_extended_data(extended_data)?;
        let buf = Symbol {
            name: address,
            extended_data,
            flags,
            stype,
            register
        };
        self.symbols.push(buf);
        self.num_symbols += 1;
        self.patch_extended_data();
        Ok(())
    }

    /// Adds a shader into this BPXS.
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
    pub fn add_shader(&mut self, shader: Shader) -> Result<(), WriteError>
    {
        let section = self.container.create_section(
            SectionHeaderBuilder::new()
                .with_type(SECTION_TYPE_SHADER)
                .with_checksum(Checksum::Crc32)
                .with_compression(CompressionMethod::Xz)
                .with_size(shader.data.len() as u32 + 1)
        );
        let mut section = self.container.get_mut(section);
        let mut buf = shader.data;
        match shader.stage {
            Stage::Vertex => buf.insert(0, 0x0),
            Stage::Hull => buf.insert(0, 0x1),
            Stage::Domain => buf.insert(0, 0x2),
            Stage::Geometry => buf.insert(0, 0x3),
            Stage::Pixel => buf.insert(0, 0x4)
        };
        section.open().ok_or(WriteError::SectionNotLoaded)?.write_all(&buf)?;
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), WriteError>
    {
        {
            let mut section = self.container.get_mut(self.symbol_table);
            let data = section.open().ok_or(WriteError::SectionNotLoaded)?;
            data.seek(SeekFrom::Start(0))?;
            for v in &self.symbols {
                v.write(data)?;
            }
        }
        self.container.save()?;
        Ok(())
    }
}

fn get_target_type_from_code(acode: u8, tcode: u8) -> Result<(Target, Type), ReadError>
{
    let target;
    let btype;

    match acode {
        0x1 => target = Target::DX11,
        0x2 => target = Target::DX12,
        0x3 => target = Target::GL33,
        0x4 => target = Target::GL40,
        0x5 => target = Target::GL41,
        0x6 => target = Target::GL42,
        0x7 => target = Target::GL43,
        0x8 => target = Target::GL44,
        0x9 => target = Target::GL45,
        0xA => target = Target::GL46,
        0xB => target = Target::ES30,
        0xC => target = Target::ES31,
        0xD => target = Target::ES32,
        0xE => target = Target::VK10,
        0xF => target = Target::VK11,
        0x10 => target = Target::VK12,
        0x11 => target = Target::MT,
        0xFF => target = Target::Any,
        _ => return Err(ReadError::InvalidCode(InvalidCodeContext::Target, acode))
    }
    if tcode == b'A' {
        //Rust refuses to parse match properly so use if/else-if blocks
        btype = Type::Assembly;
    } else if tcode == b'P' {
        btype = Type::Pipeline;
    } else {
        return Err(ReadError::InvalidCode(InvalidCodeContext::Type, tcode));
    }
    Ok((target, btype))
}

fn get_stage_from_code(code: u8) -> Result<Stage, ReadError>
{
    match code {
        0x0 => Ok(Stage::Vertex),
        0x1 => Ok(Stage::Hull),
        0x2 => Ok(Stage::Domain),
        0x3 => Ok(Stage::Geometry),
        0x4 => Ok(Stage::Pixel),
        _ => Err(ReadError::InvalidCode(InvalidCodeContext::Stage, code))
    }
}

fn read_symbol_table<T: Read + Seek>(container: &mut Container<T>, symbols: &mut Vec<Symbol>, num_symbols: u16, symbol_table: Handle) -> Result<ItemTable<Symbol>, ReadError>
{
    let mut section = container.get_mut(symbol_table);
    let count = section.header().csize as u32 / SIZE_SYMBOL_STRUCTURE as u32;

    if count != num_symbols as u32 {
        return Err(ReadError::Eos(EosContext::SymbolTable));
    }
    for _ in 0..count {
        let header = Symbol::read(section.load()?)?;
        symbols.push(header);
    }
    Ok(ItemTable::new(symbols.clone()))
}

impl<T: Read + Seek> ShaderPack<T>
{
    pub fn open(backend: T) -> Result<ShaderPack<T>, ReadError>
    {
        let container = Container::open(backend)?;
        if container.get_main_header().btype != b'P' {
            return Err(ReadError::BadType(container.get_main_header().btype));
        }
        if container.get_main_header().version != SUPPORTED_VERSION {
            return Err(ReadError::BadVersion(container.get_main_header().version));
        }
        let assembly_hash = LittleEndian::read_u64(&container.get_main_header().type_ext[0..8]);
        let num_symbols = LittleEndian::read_u16(&container.get_main_header().type_ext[8..10]);
        let (target, btype) = get_target_type_from_code(
            container.get_main_header().type_ext[10],
            container.get_main_header().type_ext[11]
        )?;
        let string_section = match container.find_section_by_type(SECTION_TYPE_STRING) {
            Some(v) => v,
            None => return Err(ReadError::MissingSection(Section::Strings))
        };
        let symbol_table = match container.find_section_by_type(SECTION_TYPE_SYMBOL_TABLE) {
            Some(v) => v,
            None => return Err(ReadError::MissingSection(Section::SymbolTable))
        };
        let strings = StringSection::new(string_section);
        Ok(Self {
            settings: Settings {
                assembly_hash,
                target,
                btype
            },
            num_symbols,
            symbol_table,
            strings,
            extended_data: None,
            container,
            symbols: Vec::with_capacity(num_symbols as _),
            table: None
        })
    }

    pub fn symbols(&mut self) -> Result<SymbolIter<T>, ReadError>
    {
        let table = self.table.get_or_insert_with_err(|| read_symbol_table(&mut self.container, &mut self.symbols, self.num_symbols, self.symbol_table))?;
        let iter = table.iter();
        Ok(SymbolIter {
            extended_data: &mut self.extended_data,
            container: &mut self.container,
            strings: &mut self.strings,
            iter
        })
    }

    /// Lists all shaders contained in this shader package.
    pub fn list_shaders(&self) -> Vec<Handle>
    {
        self.container.iter().filter_map(|v| {
            if v.header().btype == SECTION_TYPE_SHADER {
                Some(v.handle())
            } else {
                None
            }
        }).collect()
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
    /// An [ReadError](crate::variant::shader::error::ReadError) is returned if the shader could not be loaded.
    pub fn load_shader(&mut self, handle: Handle) -> Result<Shader, ReadError>
    {
        let mut section = self.container.get_mut(handle);
        if section.header().size < 1 {
            //We must at least find a stage byte
            return Err(ReadError::Eos(EosContext::Shader));
        }
        let mut buf = section.load()?.load_in_memory()?;
        let stage = get_stage_from_code(buf.remove(0))?;
        Ok(Shader { stage, data: buf })
    }
}
