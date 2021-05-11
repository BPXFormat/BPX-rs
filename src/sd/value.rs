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

use std::string::String;

use crate::sd::Array;
use crate::sd::Object;

#[derive(PartialEq, Clone)]
pub enum Value
{
    Null,
    Bool(bool),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float(f32),
    Double(f64),
    String(String),
    Array(Array),
    Object(Object)
}

impl From<bool> for Value
{
    fn from(v: bool) -> Self
    {
        return Value::Bool(v);
    }
}

impl From<u8> for Value
{
    fn from(v: u8) -> Self
    {
        return Value::Uint8(v);
    }
}

impl From<u16> for Value
{
    fn from(v: u16) -> Self
    {
        return Value::Uint16(v);
    }
}

impl From<u32> for Value
{
    fn from(v: u32) -> Self
    {
        return Value::Uint32(v);
    }
}

impl From<u64> for Value
{
    fn from(v: u64) -> Self
    {
        return Value::Uint64(v);
    }
}

impl From<i8> for Value
{
    fn from(v: i8) -> Self
    {
        return Value::Int8(v);
    }
}

impl From<i16> for Value
{
    fn from(v: i16) -> Self
    {
        return Value::Int16(v);
    }
}

impl From<i32> for Value
{
    fn from(v: i32) -> Self
    {
        return Value::Int32(v);
    }
}

impl From<i64> for Value
{
    fn from(v: i64) -> Self
    {
        return Value::Int64(v);
    }
}

impl From<f32> for Value
{
    fn from(v: f32) -> Self
    {
        return Value::Float(v);
    }
}

impl From<f64> for Value
{
    fn from(v: f64) -> Self
    {
        return Value::Double(v);
    }
}

impl From<&str> for Value
{
    fn from(v: &str) -> Self
    {
        return Value::String(String::from(v));
    }
}

impl From<String> for Value
{
    fn from(v: String) -> Self
    {
        return Value::String(v);
    }
}

impl From<Array> for Value
{
    fn from(v: Array) -> Self
    {
        return Value::Array(v);
    }
}

impl From<Object> for Value
{
    fn from(v: Object) -> Self
    {
        return Value::Object(v);
    }
}

impl <T: Into<Value>> From<Option<T>> for Value
{
    fn from(v: Option<T>) -> Self
    {
        if let Some(v) = v
        {
            return v.into();
        }
        return Value::Null;
    }
}

impl <T: Into<Value>> From<Vec<T>> for Value
{
    fn from(v: Vec<T>) -> Self
    {
        let mut arr = Array::new();
        for v1 in v
        {
            arr.add(v1.into());
        }
        return Value::Array(arr);
    }
}
