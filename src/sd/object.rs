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

use std::{
    collections::{hash_map::Keys, HashMap},
    ops::Index
};

use crate::{
    sd::{Array, Value},
    utils,
    Result
};

/// Represents a BPX Structured Data Object
#[derive(PartialEq, Clone)]
pub struct Object
{
    props: HashMap<u64, Value>
}

impl Object
{
    /// Creates a new object
    ///
    /// # Returns
    ///
    /// * a new BPXSD object
    pub fn new() -> Object
    {
        return Object { props: HashMap::new() };
    }

    /// Sets a property in the object
    ///
    /// # Arguments
    ///
    /// * `hash` - the BPX hash of the property
    /// * `value` - the [Value](crate::sd::Value) to set
    pub fn raw_set(&mut self, hash: u64, value: Value)
    {
        self.props.insert(hash, value);
    }

    /// Sets a property in the object
    ///
    /// # Arguments
    ///
    /// * `name` - the property name
    /// * `value` - the [Value](crate::sd::Value) to set
    pub fn set(&mut self, name: &str, value: Value)
    {
        self.raw_set(utils::hash(name), value);
    }

    /// Gets a property in the object
    ///
    /// # Arguments
    ///
    /// * `hash` - the BPX hash of the property
    ///
    /// # Returns
    ///
    /// * a reference to the [Value](crate::sd::Value)
    /// * None if the property could not be found
    pub fn raw_get(&self, hash: u64) -> Option<&Value>
    {
        return self.props.get(&hash);
    }

    /// Gets a property in the object
    ///
    /// # Arguments
    ///
    /// * `name` - the property name
    ///
    /// # Returns
    ///
    /// * a reference to the [Value](crate::sd::Value)
    /// * None if the property could not be found
    pub fn get(&self, name: &str) -> Option<&Value>
    {
        return self.raw_get(utils::hash(name));
    }

    /// Gets the length of the object
    ///
    /// # Returns
    ///
    /// * the number of properties in the object
    pub fn prop_count(&self) -> usize
    {
        return self.props.len();
    }

    /// Gets the list of all keys in the object
    ///
    /// # Returns
    ///
    /// * a set of all keys in the object
    pub fn get_keys(&self) -> Keys<'_, u64, Value>
    {
        return self.props.keys();
    }

    /// Writes the object to the given IO backend
    ///
    /// # Returns
    ///
    /// * nothing if the operation succeeded
    /// * an [Error](crate::error::Error) if the data could not be written
    pub fn write<TWrite: std::io::Write>(&self, dest: &mut TWrite) -> Result<()>
    {
        return super::encoder::write_structured_data(dest, self);
    }

    /// Reads a BPXSD object from an IO backend
    ///
    /// # Returns
    ///
    /// * the new BPXSD object if the operation succeeded
    /// * an [Error](crate::error::Error) if the data could not be read or the data was corrupt/truncated
    pub fn read<TRead: std::io::Read>(source: &mut TRead) -> Result<Object>
    {
        return super::decoder::read_structured_data(source);
    }
}

impl Index<&str> for Object
{
    type Output = Value;

    fn index(&self, name: &str) -> &Value
    {
        return &self.props.index(&utils::hash(name));
    }
}

impl Index<u64> for Object
{
    type Output = Value;

    fn index(&self, hash: u64) -> &Value
    {
        return &self.props.index(&hash);
    }
}
