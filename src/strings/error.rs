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

#[derive(Debug)]
pub enum ReadError
{
    /// Describes an utf8 decoding/encoding error.
    Utf8,

    /// Indicates the string reader has reached EOS (End Of Section) before the end of the string.
    Eos,

    /// Describes an io error.
    Io(std::io::Error),

    /// A section error.
    Section(crate::section::Error)
}

impl From<std::io::Error> for ReadError
{
    fn from(e: std::io::Error) -> Self
    {
        return ReadError::Io(e);
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
            ReadError::Utf8 => f.write_str("utf8 error"),
            ReadError::Eos => f.write_str("EOS reached before end of string"),
            ReadError::Io(e) => f.write_str(&format!("io error: {}", e)),
            ReadError::Section(e) => f.write_str(&format!("section error: {}", e))
        }
    }
}

#[derive(Debug)]
pub enum WriteError
{
    /// Describes an io error.
    Io(std::io::Error),

    /// A section error.
    Section(crate::section::Error)
}

impl From<std::io::Error> for WriteError
{
    fn from(e: std::io::Error) -> Self
    {
        return WriteError::Io(e);
    }
}

impl From<crate::section::Error> for WriteError
{
    fn from(e: crate::section::Error) -> Self
    {
        return WriteError::Section(e);
    }
}

impl Display for WriteError
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            WriteError::Io(e) => f.write_str(&format!("io error: {}", e)),
            WriteError::Section(e) => f.write_str(&format!("section error: {}", e))
        }
    }
}
