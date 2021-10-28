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

//! Contains utilities to work with the object table section.

use std::collections::HashMap;

use crate::{decoder::IoBackend, variant::package::PackageDecoder};
use crate::strings::ReadError;
use crate::variant::{BuildNamedTable, NamedTable};

/// Represents an object header as read from the package.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ObjectHeader
{
    /// The size of the object.
    pub size: u64,

    /// The pointer to the name of the object.
    pub name: u32,

    /// The start section index to the content.
    pub start: u32,

    /// The offset to the content in the start section.
    pub offset: u32
}

/// Helper class to query an object table.
pub struct ObjectTable
{
    list: Vec<ObjectHeader>,
    map: Option<HashMap<String, ObjectHeader>>
}

impl NamedTable for ObjectTable
{
    type Inner = ObjectHeader;

    fn new(list: Vec<Self::Inner>) -> Self
    {
        return ObjectTable {
            list,
            map: None
        };
    }

    fn lookup(&self, name: &str) -> Option<&Self::Inner>
    {
        if let Some(map) = &self.map {
            return map.get(name);
        } else {
            panic!("Lookup table has not yet been initialized, please call build_lookup_table");
        }
    }

    fn get_all(&self) -> &[Self::Inner]
    {
        return &self.list;
    }
}

impl<TBackend: IoBackend> BuildNamedTable<PackageDecoder<TBackend>> for ObjectTable
{
    fn build_lookup_table(&mut self, package: &mut PackageDecoder<TBackend>) -> Result<(), ReadError>
    {
        let mut map = HashMap::new();
        for v in &self.list {
            let name = String::from(package.get_object_name(v)?);
            map.insert(name, *v);
        }
        self.map = Some(map);
        return Ok(());
    }
}
