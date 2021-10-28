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

//! This module contains implementations for the standard BPX variants/types.

pub mod package;
pub mod shader;

/// Represents a named table with deferred/optional lookup table build.
pub trait BuildNamedTable<TDecoder>
where
    Self: NamedTable
{
    /// Builds the item map for easy and efficient lookup of items by name.
    ///
    /// **You must call this function before you can use lookup.**
    ///
    /// # Arguments
    ///
    /// * `package`: the decoder to load the strings from.
    ///
    /// returns: Result<(), Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::strings::Error) is returned if the strings could
    /// not be loaded.
    fn build_lookup_table(&mut self, package: &mut TDecoder) -> Result<(), crate::strings::ReadError>;
}

/// Represents a named table, currently used by both BPXP and BPXS.
pub trait NamedTable
{
    /// The inner item type.
    type Inner;

    /// Constructs a new NameTable from a list of items.
    ///
    /// # Arguments
    ///
    /// * `list`: the list of items.
    ///
    /// returns: Self
    fn new(list: Vec<Self::Inner>) -> Self;

    /// Lookup an item by its name.
    /// Returns None if the item does not exist.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the item to search for.
    ///
    /// returns: Option<&Self::Inner>
    ///
    /// # Panics
    ///
    /// Panics if the lookup table is not yet built.
    fn lookup(&self, name: &str) -> Option<&Self::Inner>;

    /// Gets all items in this table as a slice.
    fn get_all(&self) -> &[Self::Inner];
}
