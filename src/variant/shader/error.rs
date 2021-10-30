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

use crate::macros::{impl_err_conversion, named_enum, variant_error};

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

variant_error!(
    E {
        /// Reached EOS while reading a shader.
        Shader : "shader",

        /// Reached EOS while reading the symbol table.
        SymbolTable : "symbol table"
    }

    S {
        /// Missing strings section.
        Strings : "strings",

        /// Missing symbol table section.
        SymbolTable : "symbol table",

        /// Missing optional extended data section.
        ExtendedData : "extended data"
    }

    /// Represents a BPXS read error.
    R {
        /// Invalid code.
        ///
        /// # Arguments
        /// * the context.
        /// * the coding byte.
        InvalidCode(InvalidCodeContext, u8),

        /// Describes a missing section.
        MissingSection(Section),

        /// Describes an EOS (End Of Section) error while reading.
        Eos(EosContext),

        /// A strings error.
        Strings(crate::strings::ReadError),

        /// Describes a structured data error.
        Sd(crate::sd::error::ReadError)
    }

    /// Represents a BPXS write error.
    W {
        /// A strings error.
        Strings(crate::strings::WriteError),

        /// Describes a structured data error.
        Sd(crate::sd::error::WriteError)
    }
);

impl_err_conversion!(
    ReadError {
        crate::strings::ReadError => Strings,
        crate::sd::error::ReadError => Sd
    }
);

impl_err_conversion!(
    WriteError {
        crate::strings::WriteError => Strings,
        crate::sd::error::WriteError => Sd
    }
);

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
