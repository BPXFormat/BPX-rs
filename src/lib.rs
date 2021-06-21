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

//! This library is the official implementation for the [BPX](https://gitlab.com/bp3d/bpx/bpx/-/blob/master/BPX_Format.pdf) container format.

use std::vec::Vec;

pub mod variant;
pub mod builder;
mod compression;
pub mod decoder;
pub mod encoder;
pub mod error;
mod garraylen;
pub mod header;
pub mod sd;
pub mod section;
pub mod strings;
pub mod utils;

/// Represents a pointer to a section
///
/// *Allows indirect access to a given section instead of sharing mutable references in user code*
pub type SectionHandle = usize;

/// The interface implemented by both the BPX encoder and decoder
pub trait Interface
{
    /// Searches for the first section of a given variant
    ///
    /// # Arguments
    ///
    /// * `btype` - section variant byte
    ///
    /// # Returns
    ///
    /// * None if no section could be found
    /// * a handle to the section
    fn find_section_by_type(&self, btype: u8) -> Option<SectionHandle>;

    /// Searches for all sections of a given variant
    ///
    /// # Arguments
    ///
    /// * `btype` - section variant byte
    ///
    /// # Returns
    ///
    /// * a list of handles from all sections matching the given variant
    fn find_all_sections_of_type(&self, btype: u8) -> Vec<SectionHandle>;

    /// Locates a section by its index in the file
    ///
    /// # Arguments
    ///
    /// * `index` - the section index to search for
    ///
    /// # Returns
    ///
    /// * None if the section does not exist
    /// * a handle to the section
    fn find_section_by_index(&self, index: u32) -> Option<SectionHandle>;

    /// Gets the BPX section header
    ///
    /// *panics if the given section handle is invalid*
    ///
    /// # Arguments
    ///
    /// * `handle` - a handle to the section
    ///
    /// # Returns
    ///
    /// * read-only reference to the BPX section header
    fn get_section_header(&self, handle: SectionHandle) -> &header::SectionHeader;


    /// Gets the section index from a section handle
    ///
    /// *panics if the given section handle is invalid*
    ///
    /// # Arguments
    ///
    /// * `handle` - a handle to the section
    ///
    /// # Returns
    ///
    /// * the index of the section
    fn get_section_index(&self, handle: SectionHandle) -> u32;

    /// Opens a section for read and/or write
    ///
    /// *panics if the given section handle is invalid*
    ///
    /// # Arguments
    ///
    /// * `handle` - a handle to the section
    ///
    /// # Returns
    ///
    /// * reference to the section data
    fn open_section(&mut self, handle: SectionHandle) -> Result<&mut dyn section::SectionData>;

    /// Gets the BPX main header
    ///
    /// # Returns
    ///
    /// * read-only reference to the BPX main header
    fn get_main_header(&self) -> &header::MainHeader;
}

/// Represents a result from this library
///
/// *this acts as a shortcut to [Result](std::result::Result)<T, [Error](error::Error)>*
pub type Result<T> = std::result::Result<T, error::Error>;
