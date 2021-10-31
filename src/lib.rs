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

#![warn(missing_docs)]

//! This library is the official implementation for the [BPX](https://gitlab.com/bp3d/bpx/bpx/-/blob/master/BPX_Format.pdf) container format.

use std::{rc::Rc, vec::Vec};

use crate::section::AutoSection;

pub mod builder;
mod compression;
pub mod decoder;
pub mod encoder;
pub mod error;
mod garraylen;
pub mod header;
pub mod macros;
pub mod sd;
pub mod section;
pub mod strings;
pub mod utils;
pub mod variant;

/// Represents a pointer to a section.
///
/// *Allows indirect access to a given section instead of sharing mutable references in user code.*
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Handle(u32);

/// The interface implemented by both the BPX encoder and decoder.
pub trait Interface
{
    /// Searches for the first section of a given type.
    /// Returns None if no section could be found.
    ///
    /// # Arguments
    ///
    /// * `btype`: section type byte.
    ///
    /// returns: Option<Handle>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::encoder::Encoder;
    /// use bpx::Interface;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let file = Encoder::new(new_byte_buf(0)).unwrap();
    /// assert!(file.find_section_by_type(0).is_none());
    /// ```
    fn find_section_by_type(&self, btype: u8) -> Option<Handle>;

    /// Searches for all sections of a given type.
    /// Returns None if no section could be found.
    ///
    /// # Arguments
    ///
    /// * `btype`: section type byte.
    ///
    /// returns: Vec<Handle, Global>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::encoder::Encoder;
    /// use bpx::Interface;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let file = Encoder::new(new_byte_buf(0)).unwrap();
    /// assert_eq!(file.find_all_sections_of_type(0).len(), 0);
    /// ```
    fn find_all_sections_of_type(&self, btype: u8) -> Vec<Handle>;

    /// Locates a section by its index in the file.
    /// Returns None if the section does not exist.
    ///
    /// # Arguments
    ///
    /// * `index`: the section index to search for.
    ///
    /// returns: Option<Handle>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::encoder::Encoder;
    /// use bpx::Interface;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let file = Encoder::new(new_byte_buf(0)).unwrap();
    /// assert!(file.find_section_by_index(0).is_none());
    /// ```
    fn find_section_by_index(&self, index: u32) -> Option<Handle>;

    /// Returns the BPX section header of a section.
    ///
    /// # Arguments
    ///
    /// * `handle`: a handle to the section.
    ///
    /// returns: &SectionHeader
    ///
    /// # Panics
    ///
    /// Panics if the given section handle is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::encoder::Encoder;
    /// use bpx::Interface;
    /// use bpx::builder::SectionHeaderBuilder;
    /// use bpx::section::Section;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut file = Encoder::new(new_byte_buf(0)).unwrap();
    /// let section = file.create_section(SectionHeaderBuilder::new().with_type(1).build()).unwrap().clone();
    /// let header = file.get_section_header(section.handle());
    /// assert_eq!(header.btype, 1);
    /// ```
    fn get_section_header(&self, handle: Handle) -> &header::SectionHeader;

    /// Returns the section index from a section handle.
    ///
    /// # Arguments
    ///
    /// * `handle`: a handle to the section.
    ///
    /// returns: u32
    ///
    /// # Panics
    ///
    /// Panics if the given section handle is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::encoder::Encoder;
    /// use bpx::Interface;
    /// use bpx::builder::SectionHeaderBuilder;
    /// use bpx::section::Section;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut file = Encoder::new(new_byte_buf(0)).unwrap();
    /// let section = file.create_section(SectionHeaderBuilder::new().build()).unwrap().clone();
    /// assert_eq!(file.get_section_index(section.handle()), 0);
    /// ```
    fn get_section_index(&self, handle: Handle) -> u32;

    /// Gets a section for read and/or write.
    ///
    /// # Arguments
    ///
    /// * `handle`: a handle to the section.
    ///
    /// returns: Result<&mut dyn SectionData, Error>
    ///
    /// # Panics
    ///
    /// Panics if the given section handle is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::encoder::Encoder;
    /// use bpx::Interface;
    /// use bpx::builder::SectionHeaderBuilder;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut file = Encoder::new(new_byte_buf(0)).unwrap();
    /// let section = file.create_section(SectionHeaderBuilder::new().build()).unwrap();
    /// let mut data = section.open().unwrap();
    /// let buf = data.load_in_memory().unwrap();
    /// assert_eq!(buf.len(), 0);
    /// ```
    fn get_section(&self, handle: Handle) -> &Rc<AutoSection>;

    /// Returns a read-only reference to the BPX main header.
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::encoder::Encoder;
    /// use bpx::Interface;
    /// use bpx::builder::SectionHeaderBuilder;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut file = Encoder::new(new_byte_buf(0)).unwrap();
    /// let header = file.get_main_header();
    /// //Default BPX variant/type is 'P'
    /// assert_eq!(header.btype, 'P' as u8);
    /// ```
    fn get_main_header(&self) -> &header::MainHeader;
}
