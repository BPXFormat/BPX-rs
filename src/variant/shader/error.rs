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

use std::fmt::{Display, Formatter};

/// Represents the context of an EOS error.
#[derive(Debug)]
pub enum EosContext
{
    /// Reached EOS while reading a shader.
    Shader,

    /// Reached EOS while reading the symbol table.
    SymbolTable
}

impl EosContext
{
    pub fn name(&self) -> &'static str
    {
        return match self {
            EosContext::Shader => "shader",
            EosContext::SymbolTable => "symbol table"
        };
    }
}

/// Enumerates possible missing sections.
#[derive(Debug)]
pub enum Section
{
    /// Missing strings section.
    Strings,

    /// Missing symbol table section.
    SymbolTable,

    /// Missing optional extended data section.
    ExtendedData
}

impl Section
{
    pub fn name(&self) -> &'static str
    {
        return match self {
            Section::Strings => "string",
            Section::SymbolTable => "symbol table",
            Section::ExtendedData => "extended data"
        };
    }
}

/// Represents the context of an invalid code.
#[derive(Debug)]
pub enum InvalidCodeContext
{
    /// Invalid render API target code.
    Target,

    /// Invalid shader pack type code.
    Type,

    /// Invalid shader stage type code.
    Stage,

    /// Invalid symbol type code.
    SymbolType
}

impl InvalidCodeContext
{
    pub fn name(&self) -> &'static str
    {
        return match self {
            InvalidCodeContext::Target => "target",
            InvalidCodeContext::Type => "type",
            InvalidCodeContext::Stage => "stage",
            InvalidCodeContext::SymbolType => "symbol type"
        };
    }
}

/// Represents a BPXS read error.
#[derive(Debug)]
pub enum ReadError
{
    /// Low-level BPX decoder error.
    Bpx(crate::error::ReadError),

    /// Describes an io error.
    Io(std::io::Error),

    /// Describes a structured data error.
    Sd(crate::sd::ReadError),

    /// A section error.
    Section(crate::section::Error),

    /// A strings error.
    Strings(crate::strings::ReadError),

    /// Invalid code.
    ///
    /// # Arguments
    /// * the context.
    /// * the coding byte.
    InvalidCode(InvalidCodeContext, u8),

    /// Unsupported BPX version.
    BadVersion(u32),

    /// Unsupported BPX type code.
    BadType(u8),

    /// Describes a missing section.
    MissingSection(Section),

    /// Describes an EOS (End Of Section) error while reading.
    Eos(EosContext)
}

impl From<std::io::Error> for ReadError
{
    fn from(e: std::io::Error) -> Self
    {
        return ReadError::Io(e);
    }
}

impl From<crate::error::ReadError> for ReadError
{
    fn from(e: crate::error::ReadError) -> Self
    {
        return ReadError::Bpx(e);
    }
}

impl From<crate::strings::ReadError> for ReadError
{
    fn from(e: crate::strings::ReadError) -> Self
    {
        return ReadError::Strings(e);
    }
}

impl From<crate::sd::ReadError> for ReadError
{
    fn from(e: crate::sd::ReadError) -> Self
    {
        return ReadError::Sd(e);
    }
}

impl From<crate::section::Error> for ReadError
{
    fn from(e: crate::section::Error) -> Self
    {
        return ReadError::Section(e);
    }
}

impl Display for ReadError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            ReadError::Bpx(e) => f.write_str(&format!("BPX error: {}", e)),
            ReadError::Io(e) => f.write_str(&format!("io error: {}", e)),
            ReadError::Sd(e) => f.write_str(&format!("BPXSD error: {}", e)),
            ReadError::Section(e) => f.write_str(&format!("section error: {}", e)),
            ReadError::Strings(e) => f.write_str(&format!("strings error: {}", e)),
            ReadError::InvalidCode(ctx, code) => f.write_str(&format!("invalid {} code ({})", ctx.name(), code)),
            ReadError::BadVersion(v) => f.write_str(&format!("unsupported version ({})", v)),
            ReadError::BadType(t) => f.write_str(&format!("unknown BPX type code ({})", t)),
            ReadError::MissingSection(s) => f.write_str(&format!("missing {} section", s.name())),
            ReadError::Eos(ctx) => f.write_str(&format!("got EOS while reading {}", ctx.name()))
        }
    }
}

/// Represents a BPXS write error.
#[derive(Debug)]
pub enum WriteError
{
    /// Low-level BPX encoder error.
    Bpx(crate::error::WriteError),

    /// Describes an io error.
    Io(std::io::Error),

    /// A strings error.
    Strings(crate::strings::WriteError),

    /// A section error.
    Section(crate::section::Error),

    /// Describes a structured data error.
    Sd(crate::sd::WriteError)
}

impl From<std::io::Error> for WriteError
{
    fn from(e: std::io::Error) -> Self
    {
        return WriteError::Io(e);
    }
}

impl From<crate::error::WriteError> for WriteError
{
    fn from(e: crate::error::WriteError) -> Self
    {
        return WriteError::Bpx(e);
    }
}

impl From<crate::strings::WriteError> for WriteError
{
    fn from(e: crate::strings::WriteError) -> Self
    {
        return WriteError::Strings(e);
    }
}

impl From<crate::section::Error> for WriteError
{
    fn from(e: crate::section::Error) -> Self
    {
        return WriteError::Section(e);
    }
}

impl From<crate::sd::WriteError> for WriteError
{
    fn from(e: crate::sd::WriteError) -> Self
    {
        return WriteError::Sd(e);
    }
}

impl Display for WriteError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            WriteError::Bpx(e) => f.write_str(&format!("BPX error: {}", e)),
            WriteError::Io(e) => f.write_str(&format!("io error: {}", e)),
            WriteError::Strings(e) => f.write_str(&format!("strings error: {}", e)),
            WriteError::Section(e) => f.write_str(&format!("section error: {}", e)),
            WriteError::Sd(e) => f.write_str(&format!("BPXSD error: {}", e))
        }
    }
}
