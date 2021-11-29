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

//! Utility generation macros mainly designed for BPX and IO related error types.

#[macro_export]
/// Automatically implements conversion from the given set of error types.
macro_rules! impl_err_conversion {
    ($self: ident { $($foreign: ty => $variant: ident),* }) => {
        $(
            impl From<$foreign> for $self
            {
                fn from(e: $foreign) -> Self
                {
                    return $self::$variant(e);
                }
            }
        )*
    };
}

#[macro_export]
/// Generates an enum where each variant is assigned a &'static str name.
macro_rules! named_enum {
    (
        $(#[$enum_outer:meta])* $name: ident { $($(#[$outer:meta])* $variant: ident : $namestr: expr),* }
    ) => {
        $(#[$enum_outer])*
        #[derive(Debug)]
        pub enum $name
        {
            $(
                $(#[$outer])*
                $variant
            ),*
        }

        impl $name
        {
            /// Returns the string corresponding to the current enum variant.
            pub fn name(&self) -> &'static str
            {
                return match self {
                    $(Self::$variant => $namestr),*
                };
            }
        }
    };
}

#[macro_export]
/// Generates a set of error definitions for a new BPX variant/type.
macro_rules! variant_error {
    (
        $(E { $($(#[$eos_outer:meta])* $eos: ident : $eos_name: expr),* })?
        $(S { $($(#[$sec_outer:meta])* $sec: ident : $sec_name: expr),* })?
        $(#[$router:meta])*
        R { $($(#[$rerr_outer:meta])* $rerr: ident $(($($tr: ty),*))?),* }
        $(#[$wouter:meta])*
        W { $($(#[$werr_outer:meta])* $werr: ident $(($($tw: ty),*))?),* }
    ) => {
        $(
            named_enum!(
                /// Represents the context of an EOS error.
                EosContext {
                    $($(#[$eos_outer])* $eos : $eos_name),*
                }
            );
        )?

        $(
            named_enum!(
                /// Enumerates possible missing sections.
                Section {
                    $($(#[$sec_outer])* $sec : $sec_name),*
                }
            );
        )?

        $(#[$router])*
        #[derive(Debug)]
        pub enum ReadError
        {
            /// Low-level BPX decoder error.
            Bpx(crate::core::error::ReadError),

            /// Describes an io error.
            Io(std::io::Error),

            /// Unsupported BPX version.
            BadVersion(u32),

            /// Unsupported BPX type code.
            BadType(u8),

            $(
                $(#[$rerr_outer])*
                $rerr $(($($tr),*))?
            ),*
        }

        impl_err_conversion!(
            ReadError {
                crate::core::error::ReadError => Bpx,
                std::io::Error => Io
            }
        );

        $(#[$wouter])*
        #[derive(Debug)]
        pub enum WriteError
        {
            /// Low-level BPX encoder error.
            Bpx(crate::core::error::WriteError),

            /// Describes an io error.
            Io(std::io::Error),

            $(
                $(#[$werr_outer])*
                $werr $(($($tw),*))?
            ),*
        }

        impl_err_conversion!(
            WriteError {
                crate::core::error::WriteError => Bpx,
                std::io::Error => Io
            }
        );
    };
}

pub use impl_err_conversion;
pub use named_enum;
pub use variant_error;
