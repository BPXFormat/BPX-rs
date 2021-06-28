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

//! Utilities to manipulate the content of sections

use std::{
    boxed::Box,
    io::{Read, Result, Seek, Write},
    vec::Vec
};

mod file;
mod memory;

const MEMORY_THRESHOLD: u32 = 100000000;

/// Opaque variant intended to manipulate section data in the form of standard IO operations
pub trait SectionData: Read + Write + Seek
{
    /// Loads this section into memory
    ///
    /// # Returns
    ///
    /// * a new [Vec](std::vec::Vec) of u8 binary data
    /// * an [Error](crate::error::Error) if the section could not be loaded
    fn load_in_memory(&mut self) -> Result<Vec<u8>>;

    /// Gets the current size of this section
    ///
    /// # Returns
    ///
    /// * the size in bytes of the section so far
    fn size(&self) -> usize;
}

/// Creates new section data by automatically choosing the right container given a section size
///
/// *this function is not intended for direct use*
///
/// # Arguments
///
/// * `size` - optional size of section, if None the section will automatically reallocate to fit its content
///
/// # Returns
///
/// * a [Box](std::boxed::Box) of an opaque SectionData object
/// * an [Error](std::io::Error) in case the temporary file could not be created
pub fn new_section_data(size: Option<u32>) -> Result<Box<dyn SectionData>>
{
    if let Some(s) = size {
        if s > MEMORY_THRESHOLD {
            return Ok(Box::new(file::FileBasedSection::new(tempfile::tempfile()?)));
        } else {
            return Ok(Box::new(memory::InMemorySection::new(vec![0; s as usize])));
        }
    }
    return Ok(Box::new(file::FileBasedSection::new(tempfile::tempfile()?)));
}
