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
    convert::{From, TryFrom, TryInto},
    string::String
};

use crate::sd::{Array, Object, TypeError};

/// Represents a BPXSD value
#[derive(PartialEq, Clone)]
pub enum Value
{
    /// NULL (0x0)
    Null,

    /// bool (0x1)
    Bool(bool),

    /// u8 (0x2)
    Uint8(u8),

    /// u16 (0x3)
    Uint16(u16),

    /// u32 (0x4)
    Uint32(u32),

    /// u64 (0x5)
    Uint64(u64),

    /// i8 (0x6)
    Int8(i8),

    /// i16 (0x7)
    Int16(i16),

    /// i32 (0x8)
    Int32(i32),

    /// i64 (0x9)
    Int64(i64),

    /// f32 (0xA)
    Float(f32),

    /// f64 (0xB)
    Double(f64),

    /// [String](std::string::String) (0xC)
    String(String),

    /// [Array](crate::sd::Array) (0xD)
    Array(Array),

    /// [Object](crate::sd::Object) (0xE)
    Object(Object)
}

impl Value
{
    /// Gets the variant name of this Value
    ///
    /// # Returns
    ///
    /// * a static string reference to the variant name
    pub fn get_type_name(&self) -> &'static str
    {
        return match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Uint8(_) => "uint8",
            Value::Uint16(_) => "uint16",
            Value::Uint32(_) => "uint32",
            Value::Uint64(_) => "uint64",
            Value::Int8(_) => "int8",
            Value::Int16(_) => "int16",
            Value::Int32(_) => "int32",
            Value::Int64(_) => "int64",
            Value::Float(_) => "float",
            Value::Double(_) => "double",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object"
        };
    }
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

impl<T: Into<Value>> From<Option<T>> for Value
{
    fn from(v: Option<T>) -> Self
    {
        if let Some(v) = v {
            return v.into();
        }
        return Value::Null;
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value
{
    fn from(v: Vec<T>) -> Self
    {
        let mut arr = Array::new();
        for v1 in v {
            arr.add(v1.into());
        }
        return Value::Array(arr);
    }
}

impl TryFrom<Value> for bool
{
    type Error = TypeError;

    fn try_from(v: Value) -> Result<Self, TypeError>
    {
        if let Value::Bool(v) = v {
            return Ok(v);
        }
        return Err(TypeError::new("bool", v.get_type_name()));
    }
}

impl TryFrom<Value> for u8
{
    type Error = TypeError;

    fn try_from(v: Value) -> Result<Self, TypeError>
    {
        if let Value::Uint8(v) = v {
            return Ok(v);
        }
        return Err(TypeError::new("uint8", v.get_type_name()));
    }
}

impl TryFrom<Value> for u16
{
    type Error = TypeError;

    fn try_from(v: Value) -> Result<Self, TypeError>
    {
        return match v {
            Value::Uint16(v) => Ok(v),
            Value::Uint8(v) => Ok(v as u16),
            _ => Err(TypeError::new("uint8 or uint16", v.get_type_name()))
        };
    }
}

impl TryFrom<Value> for u32
{
    type Error = TypeError;

    fn try_from(v: Value) -> Result<Self, TypeError>
    {
        return match v {
            Value::Uint32(v) => Ok(v),
            Value::Uint16(v) => Ok(v as u32),
            Value::Uint8(v) => Ok(v as u32),
            _ => Err(TypeError::new("uint8, uint16 or uint32", v.get_type_name()))
        };
    }
}

impl TryFrom<Value> for u64
{
    type Error = TypeError;

    fn try_from(v: Value) -> Result<Self, TypeError>
    {
        return match v {
            Value::Uint64(v) => Ok(v),
            Value::Uint32(v) => Ok(v as u64),
            Value::Uint16(v) => Ok(v as u64),
            Value::Uint8(v) => Ok(v as u64),
            _ => Err(TypeError::new("uint8, uint16, uint32 or uint64", v.get_type_name()))
        };
    }
}

impl TryFrom<Value> for i8
{
    type Error = TypeError;

    fn try_from(v: Value) -> Result<Self, TypeError>
    {
        if let Value::Int8(v) = v {
            return Ok(v);
        }
        return Err(TypeError::new("int8", v.get_type_name()));
    }
}

impl TryFrom<Value> for i16
{
    type Error = TypeError;

    fn try_from(v: Value) -> Result<Self, TypeError>
    {
        return match v {
            Value::Int16(v) => Ok(v),
            Value::Int8(v) => Ok(v as i16),
            _ => Err(TypeError::new("int8 or int16", v.get_type_name()))
        };
    }
}

impl TryFrom<Value> for i32
{
    type Error = TypeError;

    fn try_from(v: Value) -> Result<Self, TypeError>
    {
        return match v {
            Value::Int32(v) => Ok(v),
            Value::Int16(v) => Ok(v as i32),
            Value::Int8(v) => Ok(v as i32),
            _ => Err(TypeError::new("int8, int16 or int32", v.get_type_name()))
        };
    }
}

impl TryFrom<Value> for i64
{
    type Error = TypeError;

    fn try_from(v: Value) -> Result<Self, TypeError>
    {
        return match v {
            Value::Int64(v) => Ok(v),
            Value::Int32(v) => Ok(v as i64),
            Value::Int16(v) => Ok(v as i64),
            Value::Int8(v) => Ok(v as i64),
            _ => Err(TypeError::new("int8, int16, int32 or int64", v.get_type_name()))
        };
    }
}

impl TryFrom<Value> for f32
{
    type Error = TypeError;

    fn try_from(v: Value) -> Result<Self, TypeError>
    {
        if let Value::Float(v) = v {
            return Ok(v);
        }
        return Err(TypeError::new("float", v.get_type_name()));
    }
}

impl TryFrom<Value> for f64
{
    type Error = TypeError;

    fn try_from(v: Value) -> Result<Self, TypeError>
    {
        return match v {
            Value::Double(v) => Ok(v),
            Value::Float(v) => Ok(v as f64),
            _ => Err(TypeError::new("float or double", v.get_type_name()))
        };
    }
}

impl TryFrom<Value> for String
{
    type Error = TypeError;

    fn try_from(v: Value) -> Result<Self, TypeError>
    {
        if let Value::String(v) = v {
            return Ok(v);
        }
        return Err(TypeError::new("string", v.get_type_name()));
    }
}

impl TryFrom<Value> for Array
{
    type Error = TypeError;

    fn try_from(v: Value) -> Result<Self, TypeError>
    {
        if let Value::Array(v) = v {
            return Ok(v);
        }
        return Err(TypeError::new("array", v.get_type_name()));
    }
}

impl TryFrom<Value> for Object
{
    type Error = TypeError;

    fn try_from(v: Value) -> Result<Self, TypeError>
    {
        if let Value::Object(v) = v {
            return Ok(v);
        }
        return Err(TypeError::new("object", v.get_type_name()));
    }
}

impl TryFrom<&Value> for bool
{
    type Error = TypeError;

    fn try_from(v: &Value) -> Result<Self, TypeError>
    {
        if let Value::Bool(v) = v {
            return Ok(*v);
        }
        return Err(TypeError::new("bool", v.get_type_name()));
    }
}

impl TryFrom<&Value> for u8
{
    type Error = TypeError;

    fn try_from(v: &Value) -> Result<Self, TypeError>
    {
        if let Value::Uint8(v) = v {
            return Ok(*v);
        }
        return Err(TypeError::new("uint8", v.get_type_name()));
    }
}

impl TryFrom<&Value> for u16
{
    type Error = TypeError;

    fn try_from(v: &Value) -> Result<Self, TypeError>
    {
        return match v {
            Value::Uint16(v) => Ok(*v),
            Value::Uint8(v) => Ok(*v as u16),
            _ => Err(TypeError::new("uint8 or uint16", v.get_type_name()))
        };
    }
}

impl TryFrom<&Value> for u32
{
    type Error = TypeError;

    fn try_from(v: &Value) -> Result<Self, TypeError>
    {
        return match v {
            Value::Uint32(v) => Ok(*v),
            Value::Uint16(v) => Ok(*v as u32),
            Value::Uint8(v) => Ok(*v as u32),
            _ => Err(TypeError::new("uint8, uint16 or uint32", v.get_type_name()))
        };
    }
}

impl TryFrom<&Value> for u64
{
    type Error = TypeError;

    fn try_from(v: &Value) -> Result<Self, TypeError>
    {
        return match v {
            Value::Uint64(v) => Ok(*v),
            Value::Uint32(v) => Ok(*v as u64),
            Value::Uint16(v) => Ok(*v as u64),
            Value::Uint8(v) => Ok(*v as u64),
            _ => Err(TypeError::new("uint8, uint16, uint32 or uint64", v.get_type_name()))
        };
    }
}

impl TryFrom<&Value> for i8
{
    type Error = TypeError;

    fn try_from(v: &Value) -> Result<Self, TypeError>
    {
        if let Value::Int8(v) = v {
            return Ok(*v);
        }
        return Err(TypeError::new("int8", v.get_type_name()));
    }
}

impl TryFrom<&Value> for i16
{
    type Error = TypeError;

    fn try_from(v: &Value) -> Result<Self, TypeError>
    {
        return match v {
            Value::Int16(v) => Ok(*v),
            Value::Int8(v) => Ok(*v as i16),
            _ => Err(TypeError::new("int8 or int16", v.get_type_name()))
        };
    }
}

impl TryFrom<&Value> for i32
{
    type Error = TypeError;

    fn try_from(v: &Value) -> Result<Self, TypeError>
    {
        return match v {
            Value::Int32(v) => Ok(*v),
            Value::Int16(v) => Ok(*v as i32),
            Value::Int8(v) => Ok(*v as i32),
            _ => Err(TypeError::new("int8, int16 or int32", v.get_type_name()))
        };
    }
}

impl TryFrom<&Value> for i64
{
    type Error = TypeError;

    fn try_from(v: &Value) -> Result<Self, TypeError>
    {
        return match v {
            Value::Int64(v) => Ok(*v),
            Value::Int32(v) => Ok(*v as i64),
            Value::Int16(v) => Ok(*v as i64),
            Value::Int8(v) => Ok(*v as i64),
            _ => Err(TypeError::new("int8, int16, int32 or int64", v.get_type_name()))
        };
    }
}

impl TryFrom<&Value> for f32
{
    type Error = TypeError;

    fn try_from(v: &Value) -> Result<Self, TypeError>
    {
        if let Value::Float(v) = v {
            return Ok(*v);
        }
        return Err(TypeError::new("float", v.get_type_name()));
    }
}

impl TryFrom<&Value> for f64
{
    type Error = TypeError;

    fn try_from(v: &Value) -> Result<Self, TypeError>
    {
        return match v {
            Value::Double(v) => Ok(*v),
            Value::Float(v) => Ok(*v as f64),
            _ => Err(TypeError::new("float or double", v.get_type_name()))
        };
    }
}

impl<'a> TryFrom<&'a Value> for &'a str
{
    type Error = TypeError;

    fn try_from(v: &'a Value) -> Result<Self, TypeError>
    {
        if let Value::String(v) = v {
            return Ok(&v);
        }
        return Err(TypeError::new("string", v.get_type_name()));
    }
}

impl<'a> TryFrom<&'a Value> for &'a Array
{
    type Error = TypeError;

    fn try_from(v: &'a Value) -> Result<Self, TypeError>
    {
        if let Value::Array(v) = v {
            return Ok(&v);
        }
        return Err(TypeError::new("array", v.get_type_name()));
    }
}

impl<'a> TryFrom<&'a Value> for &'a Object
{
    type Error = TypeError;

    fn try_from(v: &'a Value) -> Result<Self, TypeError>
    {
        if let Value::Object(v) = v {
            return Ok(v);
        }
        return Err(TypeError::new("object", v.get_type_name()));
    }
}

macro_rules! generate_option_try_from {
    ($($t:ident)*) => {
        $(
            impl TryFrom<Value> for Option<$t>
            {
                type Error = TypeError;

                fn try_from(v: Value) -> Result<Self, TypeError>
                {
                    if let Value::Null = v
                    {
                        return Ok(None);
                    }
                    let v = v.try_into()?;
                    return Ok(Some(v));
                }
            }
        )*
    };
}

macro_rules! generate_option_try_from_ref {
    ($($t:ident)*) => {
        $(
            impl <'a> TryFrom<&'a Value> for Option<&'a $t>
            {
                type Error = TypeError;

                fn try_from(v: &'a Value) -> Result<Self, TypeError>
                {
                    if let Value::Null = v
                    {
                        return Ok(None);
                    }
                    let v = v.try_into()?;
                    return Ok(Some(v));
                }
            }
        )*
    };
}

macro_rules! generate_option_try_from_ref_scalar {
    ($($t:ident)*) => {
        $(
            impl <'a> TryFrom<&'a Value> for Option<$t>
            {
                type Error = TypeError;

                fn try_from(v: &'a Value) -> Result<Self, TypeError>
                {
                    if let Value::Null = v
                    {
                        return Ok(None);
                    }
                    let v = v.try_into()?;
                    return Ok(Some(v));
                }
            }
        )*
    };
}

generate_option_try_from! {
    u8 u16 u32 u64
    i8 i16 i32 i64
    f32 f64 bool
    String Array Object
}

generate_option_try_from_ref! {
    Array Object str
}

generate_option_try_from_ref_scalar! {
    u8 u16 u32 u64
    i8 i16 i32 i64
    f32 f64 bool
}
