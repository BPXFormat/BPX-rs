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

/// Represents a structured data write error
#[derive(Debug)]
pub enum WriteError
{
    /// Describes an io error.
    ///
    /// # Arguments
    /// * the error that occured.
    Io(std::io::Error),

    /// Describes too many props or values attempted to be written as part of
    /// an Object or Array (Structured Data) (ie exceeds 255).
    ///
    /// # Arguments
    /// * actual count of props.
    PropCountExceeded(usize)
}

impl From<std::io::Error> for WriteError
{
    fn from(e: std::io::Error) -> Self
    {
        return WriteError::Io(e);
    }
}

/// Represents a structured data read error
#[derive(Debug)]
pub enum ReadError
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
}

impl From<std::io::Error> for ReadError
{
    fn from(e: std::io::Error) -> Self
    {
        return ReadError::Io(e);
    }
}

/// Represents a structured data debug error
#[derive(Debug)]
pub enum DebugError
{
    /// Indicates the object is missing a __debug__ property.
    MissingProp,

    /// Indicates the type of a value in the debugger is incorrect.
    Type(TypeError)
}

impl From<TypeError> for DebugError
{
    fn from(e: TypeError) -> Self
    {
        return DebugError::Type(e);
    }
}

/// Represents a structured data value conversion error
#[derive(Debug)]
pub struct TypeError
{
    /// The expected type name
    pub expected_type_name: &'static str,

    /// The actual type name
    pub actual_type_name: &'static str
}

impl TypeError
{
    pub fn new(expected: &'static str, actual: &'static str) -> TypeError
    {
        return TypeError {
            expected_type_name: expected,
            actual_type_name: actual
        };
    }
}
