// Copyright (c) 2022, BlockProject 3D
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
use elsa::FrozenMap;
use crate::core::{Container, Handle, SectionData};
use crate::core::builder::{Checksum, CompressionMethod, SectionHeaderBuilder};
use crate::shader::error::{EosContext, Error, Section};
use crate::shader::{SECTION_TYPE_EXTENDED_DATA, SECTION_TYPE_SHADER, Shader, Stage};
use crate::shader::decoder::get_stage_from_code;
use crate::shader::symbol::{FLAG_EXTENDED_DATA, Settings, Symbol};
use crate::strings::{load_string_section, StringSection};
use crate::table::NamedItemTable;
use crate::shader::Result;

pub struct SymbolTable
{
    strings: StringSection,
    table: NamedItemTable<Symbol>,
    extended_data: Option<Handle>,
    extended_data_objs: FrozenMap<u32, Box<crate::sd::Object>>
}

impl SymbolTable
{
    pub fn new(table: NamedItemTable<Symbol>, strings: StringSection, extended_data: Option<Handle>) -> SymbolTable {
        SymbolTable {
            strings,
            table,
            extended_data,
            extended_data_objs: FrozenMap::new()
        }
    }

    pub fn iter(&self) -> std::slice::Iter<Symbol> {
        self.table.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    pub fn len(&self) -> usize {
        self.table.len()
    }

    fn write_extended_data<T>(&mut self, container: &mut Container<T>, extended_data: Option<crate::sd::Object>) -> Result<u32>
    {
        if let Some(obj) = extended_data {
            let handle = *self.extended_data.get_or_insert_with(|| {
                container.sections_mut().create(
                    SectionHeaderBuilder::new()
                        .ty(SECTION_TYPE_EXTENDED_DATA)
                        .checksum(Checksum::Crc32)
                        .compression(CompressionMethod::Zlib)
                )
            });
            let mut section = container.sections().open(handle)?;
            let offset = section.size();
            obj.write(&mut *section)?;
            return Ok(offset as u32);
        }
        Ok(0xFFFFFF)
    }

    pub fn create<T, S: Into<Settings>>(&mut self, container: &mut Container<T>, sym: S) -> Result<usize>
    {
        let settings = sym.into();
        let address = self.strings.put(container, &settings.name)?;
        let extended_data = self.write_extended_data(container, settings.extended_data)?;
        let buf = Symbol {
            name: address,
            extended_data,
            flags: settings.flags,
            ty: settings.ty,
            register: settings.register
        };
        Ok(self.table.push(settings.name, buf))
    }

    pub fn remove(&mut self, index: usize) {
        self.table.remove(index);
    }

    pub fn load_name<T: Read + Seek>(&self, container: &Container<T>, sym: &Symbol) -> Result<&str> {
        load_string_section(container, &self.strings)?;
        let name = self.table.load_name(container, &self.strings, sym)?;
        Ok(name)
    }

    pub fn find<T: Read + Seek>(&self, container: &Container<T>, name: &str) -> Result<Option<&Symbol>> {
        load_string_section(container, &self.strings)?;
        let name = self.table.find_by_name(container, &self.strings, name)?;
        Ok(name)
    }

    pub fn get(&self, index: usize) -> Option<&Symbol> {
        self.table.get(index)
    }

    pub fn load_extended_data<T: Read + Seek>(&self, container: &Container<T>, sym: &Symbol) -> Result<&crate::sd::Object> {
        if sym.flags & FLAG_EXTENDED_DATA == 0 {
            panic!("The symbol extended data is undefined.");
        }
        if self.extended_data_objs.get(&sym.extended_data).is_none() {
            let section = self.extended_data.ok_or(Error::MissingSection(Section::ExtendedData))?;
            let mut section = container.sections().load(section)?;
            section.seek(SeekFrom::Start(sym.extended_data as _))?;
            let obj = crate::sd::Object::read(&mut *section)?;
            self.extended_data_objs.insert(sym.extended_data, Box::new(obj));
        }
        //SAFETY: We already have an if block to ensure extended data is loaded.
        Ok(unsafe { self.extended_data_objs.get(&sym.extended_data).unwrap_unchecked() })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Symbol> {
        self.table.get_mut(index)
    }
}

impl<'a> IntoIterator for &'a SymbolTable
{
    type Item = &'a Symbol;
    type IntoIter = std::slice::Iter<'a, Symbol>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Immutable guard to the table of all symbols in a BPXS.
pub struct SymbolTableRef<'a, T>
{
    pub(crate) container: &'a Container<T>,
    pub(crate) table: &'a SymbolTable
}

impl<'a, T> SymbolTableRef<'a, T>
{
    /// Gets all symbols in this table.
    pub fn iter(&self) -> std::slice::Iter<Symbol> {
        self.table.iter()
    }

    /// Returns true if this table is empty.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns the number of symbols in this table.
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Gets immutable access to a symbol by its index.
    ///
    /// # Arguments
    ///
    /// * `index`: the index of the symbol.
    ///
    /// returns: Option<&Symbol>
    pub fn get(&self, index: usize) -> Option<&Symbol> {
        self.table.get(index)
    }
}

impl<'a, 'b, T> IntoIterator for &'a SymbolTableRef<'b, T>
{
    type Item = &'a Symbol;
    type IntoIter = std::slice::Iter<'a, Symbol>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: Read + Seek> SymbolTableRef<'a, T>
{
    /// Loads the name of a symbol if it's not already loaded.
    ///
    /// # Errors
    ///
    /// If the name is not already loaded, returns an [Error](crate::package::error::Error)
    /// if the section couldn't be loaded or the string couldn't be loaded.
    pub fn load_name(&self, sym: &Symbol) -> Result<&str> {
        self.table.load_name(self.container, sym)
    }

    /// Lookup a symbol by its name.
    ///
    /// Returns None if the symbol does not exist.
    ///
    /// # Arguments
    ///
    /// * `name`: the name to search for.
    ///
    /// returns: Result<Option<&ObjectHeader>>
    ///
    /// # Errors
    ///
    /// An [Error](crate::package::error::Error) is returned if the strings could not be
    /// loaded.
    pub fn find(&self, name: &str) -> Result<Option<&Symbol>> {
        self.table.find(self.container, name)
    }

    /// Loads the extended data of a symbol if it's not already loaded.
    ///
    /// # Panics
    ///
    /// Panics if the symbol doesn't define any extended data.
    ///
    /// # Errors
    ///
    /// If the [Object](crate::sd::Object) is not already loaded, returns an
    /// [Error](crate::shader::error::Error) if the section couldn't be loaded
    /// or the [Object](crate::sd::Object) couldn't be decoded.
    pub fn load_extended_data(&self, sym: &Symbol) -> Result<&crate::sd::Object> {
        self.table.load_extended_data(self.container, sym)
    }
}

/// Mutable guard to the table of all symbols in a BPXS.
pub struct SymbolTableMut<'a, T>
{
    pub(crate) container: &'a mut Container<T>,
    pub(crate) table: &'a mut SymbolTable
}

impl<'a, T> SymbolTableMut<'a, T>
{
    /// Creates a symbol into this BPXS.
    ///
    /// # Arguments
    ///
    /// * `sym`: An [Settings](crate::shader::symbol::Settings), see [Builder](crate::shader::symbol::Builder) for more information.
    ///
    /// returns: Result<()>
    ///
    /// # Errors
    ///
    /// An [Error](crate::shader::error::Error) is returned if the symbol could not be
    /// written.
    pub fn create<S: Into<Settings>>(&mut self, sym: S) -> Result<usize> {
        self.table.create(self.container, sym)
    }

    /// Removes a symbol from this shader pack.
    ///
    /// # Arguments
    ///
    /// * `index`: the index of the symbol in the table to remove.
    pub fn remove(&mut self, index: usize) {
        self.table.remove(index)
    }

    /// Gets mutable access to a symbol by its index.
    ///
    /// # Safety
    ///
    /// This function may cause corrupted and/or non BPX compliant data to be written in the end
    /// BPX Container if the following is not respected:
    /// - When patching the extended data pointer or string pointer, it must still point to a valid
    ///   offset in the corresponding section otherwise this implementation may panic or error
    ///   when attempting to read back the container.
    /// - When patching the register number, all shaders in the package referencing this symbol
    ///   must all be re-built according to the new register number or UB may occur on some GPU
    ///   driver implementation(s).
    ///
    /// The function doesn't directly cause any UB in main program memory (which doesn't qualify as
    /// "unsafe"), however this function may indirectly cause UB on GPU shaders and/or on certain
    /// GPU driver implementations.
    ///
    /// # Arguments
    ///
    /// * `index`: the index of the symbol.
    ///
    /// returns: Option<&mut Symbol>
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Symbol> {
        self.table.get_mut(index)
    }
}

pub struct ShaderTable
{
    handles: Vec<Handle>,
    shaders: FrozenMap<u32, Box<Shader>>
}

impl ShaderTable
{
    pub fn new(handles: Vec<Handle>) -> ShaderTable {
        ShaderTable {
            handles,
            shaders: FrozenMap::new()
        }
    }

    pub fn iter(&self) -> std::slice::Iter<Handle> {
        self.handles.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }

    pub fn len(&self) -> usize {
        self.handles.len()
    }

    pub fn remove<T>(&self, container: &mut Container<T>, handle: &Handle) {
        if self.shaders.get(&handle.into_raw()).is_some() {
            container.sections_mut().remove(*handle);
        }
    }

    pub fn create<T>(&mut self, container: &mut Container<T>, shader: Shader) -> Result<Handle> {
        let handle = container.sections_mut().create(
            SectionHeaderBuilder::new()
                .ty(SECTION_TYPE_SHADER)
                .checksum(Checksum::Crc32)
                .compression(CompressionMethod::Xz)
                .size(shader.data.len() as u32 + 1)
        );
        let mut section = container.sections().open(handle)?;
        let mut buf = shader.data.clone();
        match shader.stage {
            Stage::Vertex => buf.insert(0, 0x0),
            Stage::Hull => buf.insert(0, 0x1),
            Stage::Domain => buf.insert(0, 0x2),
            Stage::Geometry => buf.insert(0, 0x3),
            Stage::Pixel => buf.insert(0, 0x4)
        };
        section.write_all(&buf)?;
        self.shaders.insert(handle.into_raw(), Box::new(shader));
        self.handles.push(handle);
        Ok(handle)
    }

    pub fn load<T: Read + Seek>(&self, container: &Container<T>, handle: &Handle) -> Result<&Shader> {
        let h = handle.into_raw();
        if self.shaders.get(&h).is_none() {
            let sections = container.sections();
            //let mut section = self.container.sections().open(handle)?;
            if sections.header(*handle).size < 1 {
                //We must at least find a stage byte
                return Err(Error::Eos(EosContext::Shader));
            }
            let mut buf = sections.load(*handle)?.load_in_memory()?;
            let stage = get_stage_from_code(buf.remove(0))?;
            let shader = Shader { stage, data: buf };
            self.shaders.insert(h, Box::new(shader));
        }
        //SAFETY: We already have an if block to ensure shader is loaded.
        Ok(unsafe { self.shaders.get(&h).unwrap_unchecked() })
    }
}

impl<'a> IntoIterator for &'a ShaderTable
{
    type Item = &'a Handle;
    type IntoIter = std::slice::Iter<'a, Handle>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Immutable guard to the table of all shaders in a BPXS.
pub struct ShaderTableRef<'a, T>
{
    pub(crate) container: &'a Container<T>,
    pub(crate) table: &'a ShaderTable
}

impl<'a, T> ShaderTableRef<'a, T>
{
    /// Gets all shaders in this table.
    pub fn iter(&self) -> std::slice::Iter<Handle> {
        self.table.iter()
    }

    /// Returns true if this table is empty.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Returns the number of shaders in this table.
    pub fn len(&self) -> usize {
        self.table.len()
    }
}

impl<'a, 'b, T> IntoIterator for &'a ShaderTableRef<'b, T>
{
    type Item = &'a Handle;
    type IntoIter = std::slice::Iter<'a, Handle>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: Read + Seek> ShaderTableRef<'a, T>
{
    /// Loads a shader into memory.
    ///
    /// # Arguments
    ///
    /// * `handle`: a handle to the shader section.
    ///
    /// returns: Result<Shader>
    ///
    /// # Errors
    ///
    /// An [Error](crate::shader::error::Error) is returned if the shader could not be loaded.
    pub fn load(&self, handle: &Handle) -> Result<&Shader> {
        self.table.load(self.container, handle)
    }
}

/// Mutable guard to the table of all shaders in a BPXS.
pub struct ShaderTableMut<'a, T>
{
    pub(crate) container: &'a mut Container<T>,
    pub(crate) table: &'a mut ShaderTable
}

impl<'a, T> ShaderTableMut<'a, T>
{
    /// Creates a shader into this BPXS.
    ///
    /// # Arguments
    ///
    /// * `shader`: the [Shader](crate::shader::Shader) to write.
    ///
    /// returns: Result<Handle, Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::shader::error::Error) is returned if the shader could not be
    /// written.
    pub fn create(&mut self, shader: Shader) -> Result<Handle> {
        self.table.create(self.container, shader)
    }

    /// Removes a shader from this shader pack.
    ///
    /// # Arguments
    ///
    /// * `handle`: the handle of the shader section to remove.
    pub fn remove(&mut self, handle: &Handle) {
        self.table.remove(self.container, handle);
    }
}
