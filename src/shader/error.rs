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

//! BPXS error definitions.

use std::fmt::{Display, Formatter};

use crate::macros::{impl_err_conversion, named_enum};

named_enum!(
    /// Represents the context of an invalid code.
    InvalidCodeContext {
        /// Invalid render API target code.
        Target: "target",

        /// Invalid shader pack type code.
        Type: "type",

        /// Invalid shader stage type code.
        Stage: "stage",

        /// Invalid symbol type code.
        SymbolType: "symbol type"
    }
);

named_enum!(
    /// Enumerates possible missing sections.
    Section {
        /// Missing strings section.
        Strings: "strings",

        /// Missing symbol table section.
        SymbolTable: "symbol table",

        /// Missing optional extended data section.
        ExtendedData: "extended data"
    }
);

named_enum!(
    /// Represents the context of an EOS error.
    EosContext {
        /// Reached EOS while reading a shader.
        Shader: "shader",

        /// Reached EOS while reading the symbol table.
        SymbolTable: "symbol table"
    }
);

/// Represents a BPXS error.
#[derive(Debug)]
pub enum Error {
    /// Low-level BPX error.
    Bpx(crate::core::error::Error),

    /// Describes an io error.
    Io(std::io::Error),

    /// Unsupported BPX version.
    BadVersion {
        /// Actual version number.
        actual: u32,

        /// Supported version number.
        supported: u32,
    },

    /// Unsupported BPX type code.
    BadType {
        /// Actual type code.
        actual: u8,

        /// Expected type code.
        expected: u8,
    },

    /// Invalid code.
    InvalidCode {
        /// The error context.
        context: InvalidCodeContext,

        /// The invalid coding byte.
        code: u8,
    },

    /// Describes a missing required section.
    MissingSection(Section),

    /// Describes an EOS (End Of Section) error while reading some item.
    Eos(EosContext),

    /// A strings error.
    Strings(crate::strings::Error),

    /// Describes a structured data error.
    Sd(crate::sd::error::Error),

    /// Indicates a section couldn't open.
    Open(crate::core::error::OpenError),
}

impl_err_conversion!(
    Error {
        crate::core::error::Error => Bpx,
        std::io::Error => Io,
        crate::sd::error::Error => Sd,
        crate::strings::Error => Strings,
        crate::core::error::OpenError => Open
    }
);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Bpx(e) => write!(f, "BPX error: {}", e),
            Error::Io(e) => write!(f, "io error: {}", e),
            Error::BadVersion { actual, supported } => write!(
                f,
                "unsupported version {} ({} supported)",
                actual, supported
            ),
            Error::BadType { actual, expected } => write!(
                f,
                "unknown BPX type code (expected {}, got {})",
                expected, actual
            ),
            Error::InvalidCode { context, code } => {
                write!(f, "invalid code {} for context '{}'", code, context.name())
            },
            Error::MissingSection(s) => write!(f, "missing {} section", s.name()),
            Error::Eos(ctx) => write!(f, "got EOS while reading {}", ctx.name()),
            Error::Sd(e) => write!(f, "BPXSD error: {}", e),
            Error::Strings(e) => write!(f, "strings error: {}", e),
            Error::Open(e) => write!(f, "section open error ({})", e),
        }
    }
}

impl std::error::Error for Error {}
