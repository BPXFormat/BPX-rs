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

//! Utilities to manipulate the content of sections.

mod auto;
mod data;

use std::fmt::{Display, Formatter};

use data::new_section_data;
pub use data::SectionData;

use crate::macros::impl_err_conversion;

/// Represents a section error.
#[derive(Debug)]
pub enum Error
{
    /// The section is already open.
    AlreadyOpen,

    /// Describes an io error.
    Io(std::io::Error)
}

impl_err_conversion!(Error { std::io::Error => Io });

impl Display for Error
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        match self {
            Error::AlreadyOpen => f.write_str("section is already open"),
            Error::Io(e) => f.write_str(&format!("io error ({})", e))
        }
    }
}

/// Trait to define basic functionality of a section content.
pub trait Section
{
    /// Returns the size of the section (without opening the section).
    fn size(&self) -> usize;

    /// Reallocates the section.
    ///
    /// # Arguments
    ///
    /// * `size`: new section size.
    ///
    /// returns: Result<Box<dyn SectionData, Global>, Error>
    ///
    /// # Errors
    ///
    /// Returns an [Error](crate::section::Error) if the section is already open or if
    /// the temporary file creation has failed.
    fn realloc(&self, size: u32) -> Result<Box<dyn SectionData>, Error>;

    /// Returns the handle of this section.
    fn handle(&self) -> SectionHandle;
}

pub use auto::AutoSection;

use crate::SectionHandle;
