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

use std::{
    io::{Read, Seek, SeekFrom, Write},
    ops::Deref,
    slice::Iter
};

use byteorder::{ByteOrder, LittleEndian};

use crate::{
    core::{
        builder::{Checksum, CompressionMethod, MainHeaderBuilder, SectionHeaderBuilder},
        header::{Struct, SECTION_TYPE_STRING},
        Container,
        SectionData
    },
    sd::Object,
    shader::{
        decoder::{get_stage_from_code, get_target_type_from_code, read_symbol_table},
        encoder::get_type_ext,
        error::{EosContext, ReadError, Section, WriteError},
        symbol::{Symbol, FLAG_EXTENDED_DATA},
        Settings,
        Shader,
        Stage,
        Target,
        Type,
        SECTION_TYPE_EXTENDED_DATA,
        SECTION_TYPE_SHADER,
        SECTION_TYPE_SYMBOL_TABLE,
        SUPPORTED_VERSION
    },
    strings::{load_string_section, StringSection},
    table::ItemTable,
    utils::OptionExtension,
    Handle
};
use crate::shader::symbol::OwnedSymbol;

/// Represents a symbol reference.
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
    /// Loads the name of this symbol if it's not already loaded.
    ///
    /// # Errors
    ///
    /// If the name is not already loaded, returns a [ReadError](crate::shader::error::ReadError)
    /// if the section couldn't be loaded or the string couldn't be loaded.
    pub fn load_name(&mut self) -> Result<&str, ReadError>
    {
        load_string_section(self.container, self.strings)?;
        let addr = self.name;
        let str = self.strings.get(self.container, addr)?;
        Ok(str)
    }

    /// Loads the extended data of this symbol if it's not already loaded.
    ///
    /// # Errors
    ///
    /// If the [Object](crate::sd::Object) is not already loaded, returns a
    /// [ReadError](crate::shader::error::ReadError) if the section couldn't be loaded
    /// or the [Object](crate::sd::Object) couldn't be decoded.
    pub fn load_extended_data(&mut self) -> Result<Object, ReadError>
    {
        if self.flags & FLAG_EXTENDED_DATA == 0 {
            panic!("The symbol extended data is undefined.");
        }
        let section = *self.extended_data.get_or_insert_with_err(|| {
            match self
                .container
                .find_section_by_type(SECTION_TYPE_EXTENDED_DATA)
            {
                Some(v) => Ok(v),
                None => Err(ReadError::MissingSection(Section::ExtendedData))
            }
        })?;
        let mut section = self.container.get_mut(section);
        let data = section.load()?;
        data.seek(SeekFrom::Start(self.sym.extended_data as _))?;
        let obj = Object::read(data)?;
        Ok(obj)
    }
}

/// An iterator over [SymbolRef](crate::shader::SymbolRef).
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
/// use std::io::{Seek, SeekFrom};
/// use bpx::shader::{Builder, Shader, ShaderPack, Stage};
/// use bpx::shader::symbol;
/// use bpx::shader::symbol::FLAG_EXTENDED_DATA;
/// use bpx::utils::new_byte_buf;
///
/// let mut bpxs = ShaderPack::create(new_byte_buf(0), Builder::new());
/// bpxs.add_symbol(symbol::Builder::new("test")).unwrap();
/// bpxs.add_shader(Shader {
///     stage: Stage::Pixel,
///     data: Vec::new()
/// }).unwrap();
/// bpxs.save();
/// //Reset our bytebuf pointer to start
/// let mut bytebuf = bpxs.into_inner().into_inner();
/// bytebuf.seek(SeekFrom::Start(0)).unwrap();
/// //Attempt decoding our in-memory BPXP
/// let mut bpxs = ShaderPack::open(bytebuf).unwrap();
/// let count = bpxs.symbols().unwrap().count();
/// assert_eq!(count, 1);
/// assert_eq!(bpxs.get_symbol_count(), 1);
/// let mut sym = bpxs.symbols().unwrap().last().unwrap();
/// assert_eq!(sym.load_name().unwrap(), "test");
/// assert_eq!(sym.flags & FLAG_EXTENDED_DATA, 0);
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
        self.settings.ty
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

impl<T: Write + Seek> ShaderPack<T>
{
    /// Creates a BPX type S.
    ///
    /// # Arguments
    ///
    /// * `backend`: A [Write](std::io::Write) + [Seek](std::io::Seek) to use as backend.
    /// * `settings`: The shader package creation settings.
    ///
    /// returns: ShaderPack<T>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::shader::Builder;
    /// use bpx::shader::ShaderPack;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut bpxs = ShaderPack::create(new_byte_buf(0), Builder::new());
    /// bpxs.save();
    /// assert!(!bpxs.into_inner().into_inner().into_inner().is_empty());
    /// ```
    pub fn create<S: Into<Settings>>(backend: T, settings: S) -> ShaderPack<T>
    {
        let settings = settings.into();
        let mut container = Container::create(
            backend,
            MainHeaderBuilder::new()
                .ty(b'P')
                .type_ext(get_type_ext(&settings))
                .version(SUPPORTED_VERSION)
        );
        let string_section = container.create_section(
            SectionHeaderBuilder::new()
                .checksum(Checksum::Weak)
                .compression(CompressionMethod::Zlib)
                .ty(SECTION_TYPE_STRING)
        );
        let symbol_table = container.create_section(
            SectionHeaderBuilder::new()
                .checksum(Checksum::Weak)
                .compression(CompressionMethod::Zlib)
                .ty(SECTION_TYPE_SYMBOL_TABLE)
        );
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
            let handle = *self.extended_data.get_or_insert_with(|| {
                self.container.create_section(
                    SectionHeaderBuilder::new()
                        .ty(SECTION_TYPE_EXTENDED_DATA)
                        .checksum(Checksum::Crc32)
                        .compression(CompressionMethod::Zlib)
                )
            });
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
    /// * `name`: The name of the symbols.
    /// * `sym`: An [OwnedSymbol](crate::shader::symbol::OwnedSymbol), see [Builder](crate::shader::symbol::Builder) for more information
    ///
    /// returns: Result<(), Error>
    ///
    /// # Errors
    ///
    /// A [WriteError](crate::shader::error::WriteError) is returned if the symbol could not be
    /// written.
    pub fn add_symbol<S: Into<OwnedSymbol>>(&mut self, sym: S) -> Result<(), WriteError>
    {
        let owned = sym.into();
        let address = self.strings.put(&mut self.container, &owned.name)?;
        let extended_data = self.write_extended_data(owned.extended_data)?;
        let buf = Symbol {
            name: address,
            extended_data,
            flags: owned.flags,
            ty: owned.ty,
            register: owned.register
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
    /// * `shader`: the [Shader](crate::shader::Shader) to write.
    ///
    /// returns: Result<(), Error>
    ///
    /// # Errors
    ///
    /// A [WriteError](crate::shader::error::WriteError) is returned if the shader could not be
    /// written.
    pub fn add_shader(&mut self, shader: Shader) -> Result<(), WriteError>
    {
        let section = self.container.create_section(
            SectionHeaderBuilder::new()
                .ty(SECTION_TYPE_SHADER)
                .checksum(Checksum::Crc32)
                .compression(CompressionMethod::Xz)
                .size(shader.data.len() as u32 + 1)
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
        section
            .open()
            .ok_or(WriteError::SectionNotLoaded)?
            .write_all(&buf)?;
        Ok(())
    }

    /// Saves this shader package.
    ///
    /// # Errors
    ///
    /// Returns a [WriteError](crate::shader::error::WriteError) if some parts of this shader
    /// package couldn't be saved.
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

impl<T: Read + Seek> ShaderPack<T>
{
    /// Opens a BPX type S.
    ///
    /// # Arguments
    ///
    /// * `backend`: A [Read](std::io::Read) + [Seek](std::io::Seek) to use as backend.
    ///
    /// returns: Result<ShaderPack<T>, ReadError>
    ///
    /// # Errors
    ///
    /// A [ReadError](crate::shader::error::ReadError) is returned if some
    /// sections/headers could not be loaded.
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::shader::Builder;
    /// use bpx::shader::ShaderPack;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut bpxs = ShaderPack::create(new_byte_buf(0), Builder::new());
    /// bpxs.save();
    /// let mut buf = bpxs.into_inner().into_inner();
    /// buf.set_position(0);
    /// let mut bpxs = ShaderPack::open(buf).unwrap();
    /// assert_eq!(bpxs.symbols().unwrap().count(), 0);
    /// ```
    pub fn open(backend: T) -> Result<ShaderPack<T>, ReadError>
    {
        let container = Container::open(backend)?;
        if container.get_main_header().ty != b'P' {
            return Err(ReadError::BadType(container.get_main_header().ty));
        }
        if container.get_main_header().version != SUPPORTED_VERSION {
            return Err(ReadError::BadVersion(container.get_main_header().version));
        }
        let assembly_hash = LittleEndian::read_u64(&container.get_main_header().type_ext[0..8]);
        let num_symbols = LittleEndian::read_u16(&container.get_main_header().type_ext[8..10]);
        let (target, ty) = get_target_type_from_code(
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
                ty
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

    /// Gets an iterator over all [SymbolRef](crate::shader::SymbolRef) in this shader package.
    ///
    /// # Errors
    ///
    /// Returns a [ReadError](crate::shader::error::ReadError) if the section couldn't be loaded
    /// or if the symbol table is truncated.
    pub fn symbols(&mut self) -> Result<SymbolIter<T>, ReadError>
    {
        let table = self.table.get_or_insert_with_err(|| {
            read_symbol_table(
                &mut self.container,
                &mut self.symbols,
                self.num_symbols,
                self.symbol_table
            )
        })?;
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
        self.container
            .iter()
            .filter_map(|v| {
                if v.ty == SECTION_TYPE_SHADER {
                    Some(v.handle())
                } else {
                    None
                }
            })
            .collect()
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
    /// An [ReadError](crate::shader::error::ReadError) is returned if the shader could not be loaded.
    pub fn load_shader(&mut self, handle: Handle) -> Result<Shader, ReadError>
    {
        let mut section = self.container.get_mut(handle);
        if section.size < 1 {
            //We must at least find a stage byte
            return Err(ReadError::Eos(EosContext::Shader));
        }
        let mut buf = section.load()?.load_in_memory()?;
        let stage = get_stage_from_code(buf.remove(0))?;
        Ok(Shader { stage, data: buf })
    }
}
