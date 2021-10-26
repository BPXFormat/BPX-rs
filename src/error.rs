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
    fmt::{Display, Formatter},
    string::String
};

//ReadError, WriteError
//TODO: Implement MorphableSection which starts as InMemorySection and as size increases auto jumps to FileSection

pub enum DeflateError
{
    Memory,
    Unsupported(&'static str),
    Data,
    Unknown,

    /// Describes an io error.
    ///
    /// # Arguments
    /// * the error that occured.
    Io(std::io::Error)
}

impl From<std::io::Error> for DeflateError
{
    fn from(e: std::io::Error) -> Self
    {
        return DeflateError::Io(e);
    }
}

pub enum InflateError
{
    Memory,
    Unsupported(&'static str),
    Data,
    Unknown,

    /// Describes an io error.
    ///
    /// # Arguments
    /// * the error that occured.
    Io(std::io::Error)
}

impl From<std::io::Error> for InflateError
{
    fn from(e: std::io::Error) -> Self
    {
        return InflateError::Io(e);
    }
}

pub enum ReadError
{
    /// Describes a checksum error.
    ///
    /// # Arguments
    /// * expected checksum value.
    /// * actual checksum value.
    Checksum(u32, u32),

    /// Describes an io error.
    ///
    /// # Arguments
    /// * the error that occured.
    Io(std::io::Error),

    /// Describes a bad version error.
    ///
    /// # Arguments
    /// * the version number.
    BadVersion(u32),

    /// Describes a data corruption error, this means an impossible
    /// byte or sequence of bytes has been found.
    ///
    /// # Arguments
    /// * message.
    Corruption(String),

    /// Describes a decompression error.
    ///
    /// # Arguments
    /// * error description string.
    Inflate(InflateError)
}

impl From<std::io::Error> for ReadError
{
    fn from(e: std::io::Error) -> Self
    {
        return ReadError::Io(e);
    }
}

impl From<InflateError> for ReadError
{
    fn from(e: InflateError) -> Self
    {
        return ReadError::Inflate(e);
    }
}

pub enum WriteError
{
    /// Describes an io error.
    ///
    /// # Arguments
    /// * the error that occured.
    Io(std::io::Error),

    /// Describes a section that is too large to be written
    /// (ie exceeds 2 pow 32 / 4Gb).
    ///
    /// # Arguments
    /// * actual size of section.
    Capacity(usize),

    /// Describes a compression error.
    ///
    /// # Arguments
    /// * error description string.
    Deflate(DeflateError)
}

impl From<std::io::Error> for WriteError
{
    fn from(e: std::io::Error) -> Self
    {
        return WriteError::Io(e);
    }
}

impl From<DeflateError> for WriteError
{
    fn from(e: DeflateError) -> Self
    {
        return WriteError::Deflate(e);
    }
}

/// Represents a BPX error
#[derive(Debug)]
pub enum Error
{
    /// Describes an io error.
    ///
    /// # Arguments
    /// * the error that occured.
    Io(std::io::Error),

    /// Describes a data truncation error, this means a section or
    /// the file itself has been truncated.
    ///
    /// # Arguments
    /// * last operation name before failure.
    Truncation(&'static str),

    /// Describes a data corruption error, this means an impossible
    /// byte or sequence of bytes has been found.
    ///
    /// # Arguments
    /// * message.
    Corruption(String),

    /// Describes an utf8 decoding/encoding error.
    ///
    /// # Arguments
    /// * last operation name before failure.
    Utf8(&'static str),

    /// Describes an operation or flag that is currently unsupported.
    ///
    /// # Arguments
    /// * message.
    Unsupported(String),

    /// Describes a section that is too large to be written
    /// (ie exceeds 2 pow 32 / 4Gb).
    ///
    /// # Arguments
    /// * actual size of section.
    Capacity(usize),

    /// Describes a compression error.
    ///
    /// # Arguments
    /// * error description string.
    Deflate(&'static str),

    /// Describes a decompression error.
    ///
    /// # Arguments
    /// * error description string.
    Inflate(&'static str),

    /// Describes a generic unknown error.
    ///
    /// # Arguments
    /// * error message.
    Other(String)
}

impl From<std::io::Error> for Error
{
    fn from(e: std::io::Error) -> Self
    {
        return Error::Io(e);
    }
}

impl From<&str> for Error
{
    fn from(e: &str) -> Self
    {
        return Error::Other(String::from(e));
    }
}

impl Display for Error
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        return match self {
            /*Error::Checksum(expected, actual) => f.write_str(&format!(
                "checksum validation failed (expected {}, got {})",
                expected, actual
            )),*/
            Error::Io(e) => f.write_str(&format!("io error ({})", e)),
            /*Error::TypeError(expected, actual) => {
                f.write_str(&format!("incompatible types (expected {}, got {})", expected, actual))
            },
            Error::PropCountExceeded(v) => f.write_str(&format!("BPXSD - too many props (count {}, max is 256)", v)),
            Error::MissingProp(v) => f.write_str(&format!("BPXSD - missing property {}", v)),*/
            Error::Truncation(e) => f.write_str(&format!(
                "unexpected EOF while reading {}, are you sure the data is not truncated?",
                e
            )),
            Error::Corruption(e) => f.write_str(&format!("illegal bytes found ({})", e)),
            Error::Utf8(e) => f.write_str(&format!("utf8 decoding/encoding error in {}", e)),
            Error::Unsupported(e) => f.write_str(&format!("unsupported operation {}", e)),
            Error::Capacity(e) => f.write_str(&format!(
                "section capacity exceeded (found {} bytes, max is 2 pow 32 bytes)",
                e
            )),
            Error::Deflate(e) => f.write_str(&format!("deflate error ({})", e)),
            Error::Inflate(e) => f.write_str(&format!("inflate error ({})", e)),
            Error::Other(e) => f.write_str(&format!("{}", e))
        };
    }
}
