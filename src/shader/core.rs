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
use byteorder::{ByteOrder, LittleEndian};
use once_cell::unsync::OnceCell;

use crate::{
    core::{
        builder::{Checksum, CompressionMethod, MainHeaderBuilder, SectionHeaderBuilder},
        header::{Struct, SECTION_TYPE_STRING},
        Container
    },
    shader::{
        decoder::{get_target_type_from_code, read_symbol_table},
        encoder::get_type_ext,
        error::{ReadError, Section, WriteError},
        Settings,
        SECTION_TYPE_EXTENDED_DATA,
        SECTION_TYPE_SHADER,
        SECTION_TYPE_SYMBOL_TABLE,
        SUPPORTED_VERSION
    },
    strings::StringSection
};
use crate::core::Handle;
use crate::shader::{ShaderTableMut, ShaderTableRef, SymbolTableMut, SymbolTableRef};
use crate::shader::table::{ShaderTable, SymbolTable};
use crate::table::NamedItemTable;

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
/// {
///     let mut symbols = bpxs.symbols_mut().unwrap();
///     symbols.create(symbol::Builder::new("test")).unwrap();
/// }
/// {
///     let mut shaders = bpxs.shaders_mut();
///     shaders.create(Shader {
///         stage: Stage::Pixel,
///         data: Vec::new()
///     }).unwrap();
/// }
/// bpxs.save();
/// //Reset our bytebuf pointer to start
/// let mut bytebuf = bpxs.into_inner().into_inner();
/// bytebuf.seek(SeekFrom::Start(0)).unwrap();
/// //Attempt decoding our in-memory BPXP
/// let mut bpxs = ShaderPack::open(bytebuf).unwrap();
/// let symbols = bpxs.symbols().unwrap();
/// let shaders = bpxs.shaders();
/// assert_eq!(symbols.len(), 1);
/// let last = symbols.iter().last().unwrap();
/// assert_eq!(symbols.load_name(last).unwrap(), "test");
/// assert_eq!(last.flags & FLAG_EXTENDED_DATA, 0);
/// let shader = shaders.load(shaders.iter().last().unwrap()).unwrap();
/// assert_eq!(shader.stage, Stage::Pixel);
/// assert_eq!(shader.data.len(), 0);
/// ```
pub struct ShaderPack<T>
{
    settings: Settings,
    container: Container<T>,
    symbol_table: Handle,
    symbols: OnceCell<SymbolTable>,
    shaders: OnceCell<ShaderTable>,
    extended_data: Option<Handle>
}

impl<T> ShaderPack<T>
{
    /// Returns the shader package settings.
    pub fn get_settings(&self) -> &Settings
    {
        &self.settings
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
                .ty(b'S')
                .type_ext(get_type_ext(&settings))
                .version(SUPPORTED_VERSION)
        );
        let string_section = container.sections_mut().create(
            SectionHeaderBuilder::new()
                .checksum(Checksum::Weak)
                .compression(CompressionMethod::Zlib)
                .ty(SECTION_TYPE_STRING)
        );
        let symbol_table = container.sections_mut().create(
            SectionHeaderBuilder::new()
                .checksum(Checksum::Weak)
                .compression(CompressionMethod::Zlib)
                .ty(SECTION_TYPE_SYMBOL_TABLE)
        );
        let strings = StringSection::new(string_section);
        ShaderPack {
            container,
            settings,
            symbol_table,
            symbols: OnceCell::from(SymbolTable::new(NamedItemTable::empty(), strings, None)),
            shaders: OnceCell::from(ShaderTable::new(Vec::new())),
            extended_data: None
        }
    }

    /// Saves this shader package.
    ///
    /// # Errors
    ///
    /// Returns a [WriteError](crate::shader::error::WriteError) if some parts of this shader
    /// package couldn't be saved.
    pub fn save(&mut self) -> Result<(), WriteError>
    {
        if let Some(syms) = self.symbols.get() {
            let mut header = *self.container.get_main_header();
            LittleEndian::write_u16(&mut header.type_ext[8..10], syms.len() as u16);
            self.container.set_main_header(header);
            let mut section = self.container.sections().open(self.symbol_table)?;
            section.seek(SeekFrom::Start(0))?;
            for v in syms {
                v.write(&mut *section)?;
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
    /// let symbols = bpxs.symbols().unwrap();
    /// assert_eq!(symbols.len(), 0);
    /// ```
    pub fn open(backend: T) -> Result<ShaderPack<T>, ReadError>
    {
        let container = Container::open(backend)?;
        if container.get_main_header().ty != b'S' {
            return Err(ReadError::BadType(container.get_main_header().ty));
        }
        if container.get_main_header().version != SUPPORTED_VERSION {
            return Err(ReadError::BadVersion(container.get_main_header().version));
        }
        let assembly_hash = LittleEndian::read_u64(&container.get_main_header().type_ext[0..8]);
        let (target, ty) = get_target_type_from_code(
            container.get_main_header().type_ext[10],
            container.get_main_header().type_ext[11]
        )?;
        let symbol_table = match container.sections().find_by_type(SECTION_TYPE_SYMBOL_TABLE) {
            Some(v) => v,
            None => return Err(ReadError::MissingSection(Section::SymbolTable))
        };
        Ok(Self {
            settings: Settings {
                assembly_hash,
                target,
                ty
            },
            symbol_table,
            extended_data: container.sections().find_by_type(SECTION_TYPE_EXTENDED_DATA),
            container,
            symbols: OnceCell::new(),
            shaders: OnceCell::new()
        })
    }

    fn load_symbol_table(&self) -> Result<SymbolTable, ReadError> {
        let handle = self.container.sections().find_by_type(SECTION_TYPE_STRING).ok_or(ReadError::MissingSection(Section::Strings))?;
        let strings = StringSection::new(handle);
        let num_symbols = LittleEndian::read_u16(&self.container.get_main_header().type_ext[8..10]);
        let table = read_symbol_table(&self.container, num_symbols, self.symbol_table)?;
        Ok(SymbolTable::new(table, strings, self.extended_data))
    }

    fn load_shader_table(&self) -> ShaderTable {
        let handles = self.container.sections().iter().filter(|v| {
            self.container.sections().header(*v).ty == SECTION_TYPE_SHADER
        }).collect();
        ShaderTable::new(handles)
    }

    /// Returns a guard for immutable access to the symbol table.
    ///
    /// This will load the symbol table if it's not already loaded.
    ///
    /// # Errors
    ///
    /// A [ReadError](crate::shader::error::ReadError) is returned if the symbol table could not be
    /// loaded.
    pub fn symbols(&self) -> Result<SymbolTableRef<T>, ReadError> {
        let table = self.symbols.get_or_try_init(|| self.load_symbol_table())?;
        Ok(SymbolTableRef {
            table,
            container: &self.container
        })
    }

    /// Returns a guard for mutable access to the symbol table.
    ///
    /// This will load the symbol table if it's not already loaded.
    ///
    /// # Errors
    ///
    /// A [ReadError](crate::shader::error::ReadError) is returned if the symbol table could not be
    /// loaded.
    pub fn symbols_mut(&mut self) -> Result<SymbolTableMut<T>, ReadError> {
        if self.symbols.get_mut().is_none() {
            //SAFETY: This is safe because only ran if the cell is none.
            unsafe { self.symbols.set(self.load_symbol_table()?).unwrap_unchecked() };
        }
        Ok(SymbolTableMut {
            table: unsafe { self.symbols.get_mut().unwrap_unchecked() },
            container: &mut self.container
        })
    }

    /// Returns a guard for immutable access to the shader table.
    ///
    /// This will load the shader table if it's not already loaded.
    pub fn shaders(&self) -> ShaderTableRef<T> {
        let table = self.shaders.get_or_init(|| self.load_shader_table());
        ShaderTableRef {
            container: &self.container,
            table
        }
    }

    /// Returns a guard for mutable access to the shader table.
    ///
    /// This will load the shader table if it's not already loaded.
    pub fn shaders_mut(&mut self) -> ShaderTableMut<T> {
        if self.shaders.get_mut().is_none() {
            //SAFETY: This is safe because only ran if the cell is none.
            unsafe { self.shaders.set(self.load_shader_table()).unwrap_unchecked() };
        }
        ShaderTableMut {
            container: &mut self.container,
            table: unsafe { self.shaders.get_mut().unwrap_unchecked() },
        }
    }
}
