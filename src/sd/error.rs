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

//! BPXSD error definitions.

use std::fmt::{Display, Formatter};

use crate::macros::impl_err_conversion;

/// Represents a structured data write error
#[derive(Debug)]
pub enum WriteError {
    /// Describes an io error.
    Io(std::io::Error),

    /// Describes too large structured data Object or Array (ie exceeds 255).
    ///
    /// # Arguments
    /// * actual number of items.
    CapacityExceeded(usize),
}

impl_err_conversion!(WriteError { std::io::Error => Io });

impl Display for WriteError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::Io(e) => write!(f, "io error: {}", e),
            WriteError::CapacityExceeded(count) => {
                write!(f, "capacity exceeded ({} > 255)", count)
            },
        }
    }
}

/// Represents a structured data read error
#[derive(Debug)]
pub enum ReadError {
    /// Describes an io error.
    ///
    /// # Arguments
    /// * the error that occured.
    Io(std::io::Error),

    /// Describes a data truncation error, this means a section or
    /// the file itself has been truncated.
    ///
    /// # Arguments
    /// * failed operation name.
    Truncation(&'static str),

    /// Describes a bad type code for a value.
    ///
    /// # Arguments
    /// * the incriminated type code.
    BadTypeCode(u8),

    /// Describes an utf8 decoding/encoding error.
    Utf8,
}

impl_err_conversion!(ReadError { std::io::Error => Io });

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::Io(e) => write!(f, "io error: {}", e),
            ReadError::Truncation(typename) => write!(f, "failed to read {}", typename),
            ReadError::BadTypeCode(code) => write!(f, "unknown value type code ({})", code),
            ReadError::Utf8 => f.write_str("utf8 error"),
        }
    }
}

/// Represents a structured data value conversion error
#[derive(Debug)]
pub struct TypeError {
    /// The expected type name
    pub expected_type_name: &'static str,

    /// The actual type name
    pub actual_type_name: &'static str,
}

impl TypeError {
    /// Creates a new BPXSD value conversion type error (shorter method).
    ///
    /// # Arguments
    ///
    /// * `expected`: the expected type name.
    /// * `actual`: the actual type name.
    ///
    /// returns: TypeError
    pub fn new(expected: &'static str, actual: &'static str) -> TypeError {
        TypeError {
            expected_type_name: expected,
            actual_type_name: actual,
        }
    }
}

impl Display for TypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unsupported type conversion (expected {}, got {})",
            self.expected_type_name, self.actual_type_name
        )
    }
}
