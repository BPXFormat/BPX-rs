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

mod deserialize;
mod serialize;

use std::fmt::{Display, Formatter};

use serde::ser::StdError;

use crate::sd::error::TypeError;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum EnumSize
{
    U8,
    U16,
    U32
}

#[derive(Debug)]
pub enum Error
{
    UnsupportedType,
    TypeMismatch(TypeError),
    Message(String),
    InvalidUtf32(u32),
    MissingMapKey,
    MissingMapValue,
    InvalidMapCall,
    InvalidEnum,
    MissingVariantData,
    MissingStructKey(&'static str)
}

impl From<TypeError> for Error
{
    fn from(e: TypeError) -> Self
    {
        Self::TypeMismatch(e)
    }
}

impl Display for Error
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            Error::UnsupportedType => f.write_str("unsupported type"),
            Error::TypeMismatch(e) => write!(f, "{}", e),
            Error::Message(s) => f.write_str(s),
            Error::InvalidUtf32(v) => write!(f, "invalid utf-32 character ({})", v),
            Error::MissingMapKey => f.write_str("missing map key"),
            Error::MissingMapValue => f.write_str("missing map value"),
            Error::InvalidMapCall => f.write_str("invalid map call"),
            Error::InvalidEnum => f.write_str("invalid enum type"),
            Error::MissingVariantData => f.write_str("missing variant data"),
            Error::MissingStructKey(name) => write!(f, "missing struct key '{}'", name)
        }
    }
}

impl StdError for Error {}

pub use deserialize::Deserializer;
pub use serialize::Serializer;
