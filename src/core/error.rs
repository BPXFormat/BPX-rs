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

//! Error declarations.

use std::{
    convert::From,
    fmt::{Display, Formatter}
};

use crate::impl_err_conversion;

/// Represents a generic decompression error.
#[derive(Debug)]
pub enum DeflateError
{
    /// Memory allocation failure.
    Memory,

    /// Some requested operation wasn't supported by this build of the compression libraries.
    Unsupported(&'static str),

    /// Data error (usually shouldn't occur, might occur due to some wrong use of compression APIs).
    Data,

    /// Unknown error (low-level error from chosen compression library).
    Unknown,

    /// Describes an io error.
    Io(std::io::Error)
}

impl_err_conversion!(DeflateError { std::io::Error => Io });

impl Display for DeflateError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            DeflateError::Memory => f.write_str("memory allocation failure"),
            DeflateError::Unsupported(e) => write!(f, "unsupported operation ({})", e),
            DeflateError::Data => f.write_str("data error"),
            DeflateError::Unknown => f.write_str("low-level unknown error"),
            DeflateError::Io(e) => write!(f, "io error: {}", e)
        }
    }
}

/// Represents a generic compression error.
#[derive(Debug)]
pub enum InflateError
{
    /// Memory allocation failure.
    Memory,

    /// Some requested operation wasn't supported by this build of the compression libraries.
    Unsupported(&'static str),

    /// Data error (usually means input data is corrupted).
    Data,

    /// Unknown error (low-level error from chosen compression library).
    Unknown,

    /// Describes an io error.
    Io(std::io::Error)
}

impl_err_conversion!(InflateError { std::io::Error => Io });

impl Display for InflateError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            InflateError::Memory => f.write_str("memory allocation failure"),
            InflateError::Unsupported(e) => write!(f, "unsupported operation ({})", e),
            InflateError::Data => f.write_str("data error"),
            InflateError::Unknown => f.write_str("low-level unknown error"),
            InflateError::Io(e) => write!(f, "io error: {}", e)
        }
    }
}

/// Represents a BPX read error.
#[derive(Debug)]
pub enum ReadError
{
    /// Describes a checksum error.
    ///
    /// # Arguments
    /// * expected checksum value.
    /// * actual checksum value.
    Checksum(u32, u32),

    /// Describes an io error.
    Io(std::io::Error),

    /// Describes a bad version error.
    ///
    /// # Arguments
    /// * the incriminated version number.
    BadVersion(u32),

    /// Describes a bad signature error
    ///
    /// # Arguments
    /// * the incriminated signature.
    BadSignature([u8; 3]),

    /// Describes a decompression error.
    Inflate(InflateError),

    /// The requested section is already in use.
    ///
    /// This usually means the section is referencing itself, this error variant is intended
    /// to prevent this case.
    SectionInUse
}

impl_err_conversion!(
    ReadError {
        std::io::Error => Io,
        InflateError => Inflate
    }
);

impl Display for ReadError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            ReadError::Checksum(expected, actual) => write!(
                f,
                "checksum validation failed (expected {}, got {})",
                expected, actual
            ),
            ReadError::Io(e) => write!(f, "io error: {}", e),
            ReadError::BadVersion(v) => write!(f, "unknown file version ({})", v),
            ReadError::BadSignature(sig) => {
                write!(f, "unknown file signature ({}{}{})", sig[0], sig[1], sig[2])
            },
            ReadError::Inflate(e) => write!(f, "inflate error: {}", e),
            ReadError::SectionInUse => f.write_str("section in use")
        }
    }
}

/// Represents a BPX write error.
#[derive(Debug)]
pub enum WriteError
{
    /// Describes an io error.
    Io(std::io::Error),

    /// Describes a section that is too large to be written
    /// (ie exceeds 2 pow 32 / 4Gb).
    ///
    /// # Arguments
    /// * actual size of section.
    Capacity(usize),

    /// Describes a compression error.
    Deflate(DeflateError),

    /// A section has not yet been loaded.
    SectionNotLoaded
}

impl_err_conversion!(
    WriteError {
        std::io::Error => Io,
        DeflateError => Deflate
    }
);

impl Display for WriteError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            WriteError::Io(e) => write!(f, "io error: {}", e),
            WriteError::Capacity(size) => {
                write!(f, "maximum section size exceeded ({} > 2^32)", size)
            },
            WriteError::Deflate(e) => write!(f, "deflate error: {}", e),
            WriteError::SectionNotLoaded => f.write_str("section not loaded")
        }
    }
}

/// Represents possible errors when opening a section.
#[derive(Debug)]
pub enum OpenError
{
    /// The requested section is already in use.
    ///
    /// This usually means the section is referencing itself, this error variant is intended
    /// to prevent this case.
    SectionInUse,

    /// The requested section has not been loaded.
    SectionNotLoaded
}

impl Display for OpenError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenError::SectionInUse => f.write_str("section in use"),
            OpenError::SectionNotLoaded => f.write_str("section not loaded")
        }
    }
}
