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

use std::collections::HashMap;
use std::ops::Index;
use std::collections::hash_map::Keys;

use crate::sd::Value;
use crate::sd::Array;
use crate::utils;

#[derive(PartialEq, Clone)]
pub struct Object
{
    props: HashMap<u64, Value>,
    prop_names: Array
}

impl Object
{
    pub fn new() -> Object
    {
        return Object
        {
            props: HashMap::new(),
            prop_names: Array::new()
        }
    }

    pub fn raw_set(&mut self, hash: u64, value: Value)
    {
        self.props.insert(hash, value);
    }

    pub fn set(&mut self, name: &str, value: Value)
    {
        self.raw_set(utils::hash(name), value);
        self.prop_names.add(Value::String(String::from(name)));
    }

    pub fn raw_get(&self, hash: u64) -> Option<&Value>
    {
        return self.props.get(&hash);
    }

    pub fn get(&self, name: &str) -> Option<&Value>
    {
        return self.raw_get(utils::hash(name));
    }

    pub fn prop_count(&self) -> usize
    {
        return self.props.len();
    }

    pub fn get_keys(&self) -> Keys<'_, u64, Value>
    {
        return self.props.keys();
    }

    pub fn add_debug_info(&mut self)
    {
        let prop_names = std::mem::replace(&mut self.prop_names, Array::new());
        self.set("__debug__", Value::Array(prop_names));
    }

    pub fn write(&self, dest: &mut dyn std::io::Write) -> std::io::Result<()>
    {
        return super::encoder::write_structured_data(dest, self);
    }

    pub fn read(source: &mut dyn std::io::Read) -> std::io::Result<Object>
    {
        return super::decoder::read_structured_data(source);
    }
}

impl Index<&str> for Object
{
    type Output = Value;

    fn index<'a>(&'a self, name: &str) -> &'a Value
    {
        return &self.props.index(&utils::hash(name));
    }
}

impl Index<u64> for Object
{
    type Output = Value;

    fn index<'a>(&'a self, hash: u64) -> &'a Value
    {
        return &self.props.index(&hash);
    }
}
