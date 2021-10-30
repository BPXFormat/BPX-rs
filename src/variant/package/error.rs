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

//! BPXP error definitions.

use std::fmt::{Display, Formatter};

use crate::macros::{impl_err_conversion, named_enum, variant_error};

named_enum!(
    /// Represents the context of an invalid code.
    InvalidCodeContext {
        /// Invalid architecture code byte.
        Arch: "architecture",

        /// Invalid platform code byte.
        Platform: "platform"
    }
);

variant_error!(
    E {
        /// Reached EOS while reading an object.
        Object : "object",

        /// Reached EOS while reading the object table.
        ObjectTable : "object table"
    }

    S {
        /// Missing strings section.
        Strings : "string",

        /// Missing object table section.
        ObjectTable : "object table"
    }

    /// Represents a BPXP read error.
    R {
        /// Invalid code.
        ///
        /// # Arguments
        /// * the context.
        /// * the coding byte.
        InvalidCode(InvalidCodeContext, u8),

        /// Describes a missing required section.
        MissingSection(Section),

        /// Describes an EOS (End Of Section) error while reading some item.
        Eos(EosContext),

        /// Indicates a blank string was obtained when attempting to unpack a BPXP to the file system.
        BlankString,

        /// Describes a structured data error.
        Sd(crate::sd::error::ReadError),

        /// A strings error.
        Strings(crate::strings::ReadError)
    }

    /// Represents a BPXP write error.
    W {
        /// A strings error.
        Strings(crate::strings::WriteError),

        /// Describes a structured data error.
        Sd(crate::sd::error::WriteError),

        /// Indicates an invalid path while attempting to pack some files.
        InvalidPath
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
            ReadError::Section(e) => f.write_str(&format!("section error: {}", e)),
            ReadError::BadVersion(v) => f.write_str(&format!("unsupported version ({})", v)),
            ReadError::BadType(t) => f.write_str(&format!("unknown BPX type code ({})", t)),
            ReadError::InvalidCode(ctx, code) => f.write_str(&format!("invalid {} code ({})", ctx.name(), code)),
            ReadError::MissingSection(s) => f.write_str(&format!("missing {} section", s.name())),
            ReadError::Eos(ctx) => f.write_str(&format!("got EOS while reading {}", ctx.name())),
            ReadError::BlankString => f.write_str("blank strings are not supported when unpacking to file system"),
            ReadError::Sd(e) => f.write_str(&format!("BPXSD error: {}", e)),
            ReadError::Strings(e) => f.write_str(&format!("strings error: {}", e))
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
            WriteError::Section(e) => f.write_str(&format!("section error: {}", e)),
            WriteError::Strings(e) => f.write_str(&format!("strings error: {}", e)),
            WriteError::Sd(e) => f.write_str(&format!("BPXSD error: {}", e)),
            WriteError::InvalidPath => f.write_str("path does not contain any file name")
        }
    }
}
