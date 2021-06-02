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
    ops::{Index, IndexMut},
    vec::Vec
};

use crate::sd::Value;

/// Represents a BPX Structured Data Array
#[derive(PartialEq, Clone)]
pub struct Array
{
    data: Vec<Value>
}

impl Array
{
    /// Creates a new array
    ///
    /// # Returns
    ///
    /// * a new BPXSD array
    pub fn new() -> Array
    {
        return Array { data: Vec::new() };
    }

    /// Adds a value at the end of the array
    ///
    /// # Arguments
    ///
    /// * `v` - the [Value](crate::sd::Value) to add
    pub fn add(&mut self, v: Value)
    {
        self.data.push(v);
    }

    /// Removes a value from the array
    ///
    /// # Arguments
    ///
    /// * `pos` - the position of the item in the array to remove
    pub fn remove_at(&mut self, pos: usize)
    {
        self.data.remove(pos);
    }

    /// Removes a range of values from the array
    ///
    /// # Arguments
    ///
    /// * `item` - the [Value](crate::sd::Value) to remove
    pub fn remove(&mut self, item: Value)
    {
        for i in 0..self.data.len() {
            if self.data[i] == item {
                self.data.remove(i);
            }
        }
    }

    /// Attempts to get an item at a given position
    ///
    /// # Arguments
    ///
    /// * `pos` - the position of the item
    ///
    /// # Returns
    ///
    /// * a reference to the [Value](crate::sd::Value) at the given position
    /// * None if no value could be found at the given position
    pub fn get(&self, pos: usize) -> Option<&Value>
    {
        return self.data.get(pos);
    }

    /// Gets the length of the array
    ///
    /// # Returns
    ///
    /// * the length of the array
    pub fn len(&self) -> usize
    {
        return self.data.len();
    }
}

impl Index<usize> for Array
{
    type Output = Value;

    fn index<'a>(&'a self, i: usize) -> &'a Value
    {
        return &self.data[i];
    }
}

impl IndexMut<usize> for Array
{
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut Value
    {
        return &mut self.data[i];
    }
}
