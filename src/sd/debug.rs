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

//! Provides support for debug symbols to BPXSD object.

use std::{collections::HashMap, convert::TryInto};

use crate::{
    sd::{Array, Object},
    utils::hash
};
use crate::sd::error::TypeError;
use crate::sd::Value;

/// A BPXSD object debugger iterator.
pub struct Iter<'a>
{
    inner: crate::sd::object::Iter<'a>,
    symbols_map: &'a HashMap<u64, String>
}

impl<'a> Iterator for Iter<'a>
{
    type Item = (Option<&'a str>, u64, &'a Value);

    fn next(&mut self) -> Option<Self::Item>
    {
        let (mut k, mut v) = self.inner.next()?;
        while k == hash("__debug__") {
            let (k1, v1) = self.inner.next()?;
            k = k1;
            v = v1;
        }
        Some((self.symbols_map.get(&k).map(|v| &**v), k, v))
    }
}

/// A wrapper to BPXSD object with debugging capabilities.
#[derive(PartialEq, Clone)]
pub struct Debugger
{
    inner: Object,
    symbols_map: HashMap<u64, String>,
    symbols_list: Vec<String>
}

impl Debugger
{
    /// Attach a debugger to an object.
    ///
    /// # Arguments
    ///
    /// * `inner`: the object to attach the debugger to.
    ///
    /// returns: Result<Debugger, TypeError>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::sd::Object;
    /// use bpx::sd::Debugger;
    ///
    /// let mut obj = Debugger::attach(Object::new()).unwrap();
    /// obj.set("Test", 12.into());
    /// let inner = obj.detach();
    /// assert_eq!(inner.len(), 2);
    /// assert!(inner.get("__debug__").is_some());
    /// assert!(inner.get("Test").is_some());
    /// ```
    pub fn attach(inner: Object) -> Result<Debugger, TypeError>
    {
        let mut dbg = Debugger {
            inner,
            symbols_map: HashMap::new(),
            symbols_list: Vec::new()
        };
        if let Some(val) = dbg.inner.get("__debug__") {
            let val: &Array = val.try_into()?;
            for i in 0..val.len() {
                let sym: &str = (&val[i]).try_into()?;
                dbg.symbols_map.insert(hash(sym), sym.into());
                dbg.symbols_list.push(sym.into());
            }
        }
        Ok(dbg)
    }

    /// Performs a lookup for a given hash value in this symbol list.
    /// Returns None if the symbol does not exist.
    ///
    /// # Arguments
    ///
    /// * `hash`: the hash for which to search the symbol name.
    ///
    /// returns: Option<&str>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::sd::{Object, Debugger};
    /// use bpx::utils::hash;
    ///
    /// let debugger = Debugger::attach(Object::new()).unwrap();
    /// assert!(debugger.lookup(hash("Test")).is_none());
    /// ```
    pub fn lookup(&self, hash: u64) -> Option<&str>
    {
        if let Some(v) = self.symbols_map.get(&hash) {
            return Some(v);
        }
        None
    }

    /// Sets a property in the object.
    ///
    /// # Arguments
    ///
    /// * `name`: the property name.
    /// * `value`: the [Value](crate::sd::Value) to set.
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::sd::Object;
    /// use bpx::sd::Debugger;
    ///
    /// let mut obj = Debugger::attach(Object::new()).unwrap();
    /// assert!(obj.is_empty());
    /// obj.set("Test", 12.into());
    /// assert_eq!(obj.len(), 1);
    /// ```
    pub fn set(&mut self, name: &str, value: Value)
    {
        let hash = hash(name);
        self.inner.raw_set(hash, value);
        self.symbols_list.push(name.into());
        self.symbols_map.insert(hash, name.into());
    }

    /// Gets a property in the object.
    /// Returns None if the property name does not exist.
    ///
    /// # Arguments
    ///
    /// * `name`: the property name.
    ///
    /// returns: Option<&Value>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::sd::{Debugger, Object};
    /// use bpx::sd::Value;
    ///
    /// let mut obj = Debugger::attach(Object::new()).unwrap();
    /// obj.set("Test", 12.into());
    /// assert!(obj.get("Test").is_some());
    /// assert!(obj.get("Test1").is_none());
    /// assert!(obj.get("Test").unwrap() == &Value::from(12));
    /// ```
    pub fn get(&self, name: &str) -> Option<&Value>
    {
        self.inner.get(name)
    }

    /// Returns the number of properties in the object.
    pub fn len(&self) -> usize
    {
        self.inner.len()
    }

    /// Returns whether this object is empty
    pub fn is_empty(&self) -> bool
    {
        self.inner.is_empty()
    }

    /// Iterate through the object keys, values and names.
    pub fn iter(&self) -> Iter
    {
        Iter {
            inner: self.inner.iter(),
            symbols_map: &self.symbols_map
        }
    }

    /// Detaches the debugger from the inner object and return the inner object
    pub fn detach(mut self) -> Object
    {
        self.inner.set("__debug__", self.symbols_list.into());
        self.inner
    }
}

impl<'a> IntoIterator for &'a Debugger
{
    type Item = (Option<&'a str>, u64, &'a Value);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.iter()
    }
}
