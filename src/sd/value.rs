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

use crate::{
    macros::impl_err_conversion,
    macros::named_enum,
    sd::{error::TypeError, Array, Object}
};

named_enum!(
    /// Represents the value type.
    #[derive(Copy, Clone)]
    Type {
        /// A null type.
        Null: "null",

        /// A bool type.
        Bool: "bool",

        /// An 8 bit unsigned integer type.
        Uint8: "uint8",

        /// A 16 bit unsigned integer type.
        Uint16: "uint16",

        /// A 32 bit unsigned integer type.
        Uint32: "uint32",

        /// An 64 bit unsigned integer type.
        Uint64: "uint64",

        /// An 8 bit integer type.
        Int8: "int8",

        /// A 16 bit integer type.
        Int16: "int16",

        /// A 32 bit integer type.
        Int32: "int32",

        /// A 64 bit integer type.
        Int64: "int64",

        /// A 32 bit float type.
        Float: "float",

        /// A 64 bit float type.
        Double: "double",

        /// A string type.
        String: "string",

        /// An array type.
        Array: "array",

        /// An object type.
        Object: "object"
    }
);

/// Represents a BPXSD value.
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
    /// Gets the type of this Value.
    pub fn get_type(&self) -> Type {
        match self {
            Value::Null => Type::Null,
            Value::Bool(_) => Type::Bool,
            Value::Uint8(_) => Type::Uint8,
            Value::Uint16(_) => Type::Uint16,
            Value::Uint32(_) => Type::Uint32,
            Value::Uint64(_) => Type::Uint64,
            Value::Int8(_) => Type::Int8,
            Value::Int16(_) => Type::Int16,
            Value::Int32(_) => Type::Int32,
            Value::Int64(_) => Type::Int64,
            Value::Float(_) => Type::Float,
            Value::Double(_) => Type::Double,
            Value::String(_) => Type::String,
            Value::Array(_) => Type::Array,
            Value::Object(_) => Type::Object
        }
    }
}

impl_err_conversion!(
    Value {
        bool => Bool,
        u8 => Uint8,
        u16 => Uint16,
        u32 => Uint32,
        u64 => Uint64,
        i8 => Int8,
        i16 => Int16,
        i32 => Int32,
        i64 => Int64,
        f32 => Float,
        f64 => Double,
        String => String,
        Array => Array,
        Object => Object
    }
);

impl From<&str> for Value
{
    fn from(v: &str) -> Self
    {
        Value::String(String::from(v))
    }
}

impl<T: Into<Value>> From<Option<T>> for Value
{
    fn from(v: Option<T>) -> Self
    {
        if let Some(v) = v {
            return v.into();
        }
        Value::Null
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value
{
    fn from(v: Vec<T>) -> Self
    {
        let mut arr = Array::new();
        for v1 in v {
            arr.as_mut().push(v1.into());
        }
        Value::Array(arr)
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
        return Err(TypeError::new(Type::Bool, v.get_type()));
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
        return Err(TypeError::new(Type::Uint8, v.get_type()));
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
            _ => Err(TypeError::new(Type::Uint16, v.get_type()))
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
            _ => Err(TypeError::new(Type::Uint32, v.get_type()))
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
            _ => Err(TypeError::new(Type::Uint64, v.get_type()))
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
        return Err(TypeError::new(Type::Int8, v.get_type()));
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
            _ => Err(TypeError::new(Type::Int16, v.get_type()))
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
            _ => Err(TypeError::new(Type::Int32, v.get_type()))
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
            _ => Err(TypeError::new(Type::Int64, v.get_type()))
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
        return Err(TypeError::new(Type::Float, v.get_type()));
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
            _ => Err(TypeError::new(Type::Double, v.get_type()))
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
        return Err(TypeError::new(Type::String, v.get_type()));
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
        return Err(TypeError::new(Type::Array, v.get_type()));
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
        return Err(TypeError::new(Type::Object, v.get_type()));
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
        return Err(TypeError::new(Type::Bool, v.get_type()));
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
        return Err(TypeError::new(Type::Uint8, v.get_type()));
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
            _ => Err(TypeError::new(Type::Uint16, v.get_type()))
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
            _ => Err(TypeError::new(Type::Uint32, v.get_type()))
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
            _ => Err(TypeError::new(Type::Uint64, v.get_type()))
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
        return Err(TypeError::new(Type::Int8, v.get_type()));
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
            _ => Err(TypeError::new(Type::Int16, v.get_type()))
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
            _ => Err(TypeError::new(Type::Int32, v.get_type()))
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
            _ => Err(TypeError::new(Type::Int64, v.get_type()))
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
        return Err(TypeError::new(Type::Float, v.get_type()));
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
            _ => Err(TypeError::new(Type::Double, v.get_type()))
        };
    }
}

impl<'a> TryFrom<&'a Value> for &'a str
{
    type Error = TypeError;

    fn try_from(v: &'a Value) -> Result<Self, TypeError>
    {
        if let Value::String(v) = v {
            return Ok(v);
        }
        return Err(TypeError::new(Type::String, v.get_type()));
    }
}

impl<'a> TryFrom<&'a Value> for &'a Array
{
    type Error = TypeError;

    fn try_from(v: &'a Value) -> Result<Self, TypeError>
    {
        if let Value::Array(v) = v {
            return Ok(v);
        }
        return Err(TypeError::new(Type::Array, v.get_type()));
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
        return Err(TypeError::new(Type::Object, v.get_type()));
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
