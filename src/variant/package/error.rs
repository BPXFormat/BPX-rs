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

use crate::macros::variant_error;
use crate::macros::impl_err_conversion;
use crate::macros::named_enum;

named_enum!(
    /// Represents the context of an invalid code.
    InvalidCodeContext {
        Arch : "architecture",
        Platform : "platform"
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
        Sd(crate::sd::ReadError),

        /// A strings error.
        Strings(crate::strings::ReadError)
    }

    /// Represents a BPXP write error.
    W {
        /// A strings error.
        Strings(crate::strings::WriteError),

        /// Describes a structured data error.
        Sd(crate::sd::WriteError),

        //Indicates an invalid path while attempting to pack some files.
        InvalidPath
    }
);

impl_err_conversion!(
    ReadError {
        crate::strings::ReadError => Strings,
        crate::sd::ReadError => Sd
    }
);

impl_err_conversion!(
    WriteError {
        crate::strings::WriteError => Strings,
        crate::sd::WriteError => Sd
    }
);
