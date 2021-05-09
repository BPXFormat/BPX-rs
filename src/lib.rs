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

use std::vec::Vec;

//mod extension;
mod garraylen;
mod compression;
//pub mod bpx;
//pub mod bpxp;
pub mod section;
pub mod strings;
pub mod sd;
pub mod utils;
pub mod encoder;
pub mod decoder;
pub mod header;
pub mod builder;

pub type SectionHandle = usize;

pub trait BPX
{
    fn find_section_by_type(&self, btype: u8) -> Option<SectionHandle>;
    fn find_all_sections_of_type(&self, btype: u8) -> Vec<SectionHandle>;
    fn find_section_by_index(&self, index: u32) -> Option<SectionHandle>;
    fn get_section_header(&self, handle: SectionHandle) -> &header::SectionHeader;
    fn open_section(&mut self, handle: SectionHandle) -> std::io::Result<&mut dyn section::SectionData>;
    fn get_main_header(&self) -> &header::MainHeader;
}

pub trait OptionExtension<T>
{
    fn get_or_insert_with_err<TError, F: FnOnce() -> Result<T, TError>>(&mut self, f: F) -> Result<&mut T, TError>;
}

impl <T> OptionExtension<T> for Option<T>
{
    fn get_or_insert_with_err<TError, F: FnOnce() -> Result<T, TError>>(&mut self, f: F) -> Result<&mut T, TError>
    {
        if let None = *self {
            *self = Some(f()?);
        }
    
        match self {
            Some(v) => Ok(v),
            // SAFETY: a `None` variant for `self` would have been replaced by a `Some`
            // variant in the code above.
            None => unsafe { std::hint::unreachable_unchecked() },
        }    
    }
}
