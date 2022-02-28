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

use std::fmt::Display;

use serde::{
    ser::{
        SerializeMap,
        SerializeSeq,
        SerializeStruct,
        SerializeStructVariant,
        SerializeTuple,
        SerializeTupleStruct,
        SerializeTupleVariant
    },
    Serialize
};

use crate::sd::{
    serde::{EnumSize, Error},
    Array,
    Object,
    Value
};
use crate::sd::debug::{Debugger, ODebugger};

enum DebuggerOrObject
{
    Debugger(ODebugger),
    Object(crate::sd::Object)
}

impl DebuggerOrObject
{
    pub fn with_capacity(capacity: usize, debug: bool) -> DebuggerOrObject
    {
        if debug {
            DebuggerOrObject::Debugger(Debugger::attach(Object::with_capacity(capacity as _)).unwrap())
        } else {
            DebuggerOrObject::Object(Object::with_capacity(capacity as _))
        }
    }

    pub fn get(&self, key: &str) -> Option<&crate::sd::Value>
    {
        match self {
            DebuggerOrObject::Debugger(v) => v.as_ref().get(key),
            DebuggerOrObject::Object(v) => v.get(key)
        }
    }

    pub fn set(&mut self, key: &str, value: crate::sd::Value)
    {
        match self {
            DebuggerOrObject::Debugger(v) => v.set(key, value),
            DebuggerOrObject::Object(v) => v.set(key, value)
        }
    }

    pub fn into(self) -> crate::sd::Value
    {
        match self {
            DebuggerOrObject::Debugger(v) => v.detach().into(),
            DebuggerOrObject::Object(v) => v.into()
        }
    }
}

impl serde::ser::Error for Error
{
    fn custom<T>(msg: T) -> Self
    where
        T: Display
    {
        Error::Message(msg.to_string())
    }
}

pub struct Seq
{
    arr: Array,
    enum_size: EnumSize,
    debug: bool
}

impl Seq
{
    fn serialize<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize
    {
        self.arr.as_mut().push(value.serialize(Serializer::new(self.enum_size, self.debug))?);
        Ok(())
    }
}

impl SerializeSeq for Seq
{
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        self.serialize(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error>
    {
        Ok(self.arr.into())
    }
}

impl SerializeTuple for Seq
{
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        self.serialize(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error>
    {
        Ok(self.arr.into())
    }
}

impl SerializeTupleStruct for Seq
{
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        self.serialize(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error>
    {
        Ok(self.arr.into())
    }
}

impl SerializeTupleVariant for Seq
{
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        self.serialize(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error>
    {
        Ok(self.arr.into())
    }
}

pub struct Map
{
    cur_obj: DebuggerOrObject,
    arr: Array,
    enum_size: EnumSize,
    debug: bool
}

impl Map
{
    fn check_update(&mut self)
    {
        if self.cur_obj.get("__key__").is_some() && self.cur_obj.get("__value__").is_some() {
            let val = std::mem::replace(
                &mut self.cur_obj,
                DebuggerOrObject::with_capacity(2, self.debug)
            );
            self.arr.as_mut().push(val.into());
        }
    }
}

impl SerializeMap for Map
{
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        self.cur_obj.set(
            "__key__",
            key.serialize(Serializer::new(self.enum_size, self.debug))?
        );
        self.check_update();
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        self.cur_obj.set(
            "__value__",
            value.serialize(Serializer::new(self.enum_size, self.debug))?
        );
        self.check_update();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error>
    {
        Ok(self.arr.into())
    }
}

pub struct Struct
{
    obj: DebuggerOrObject,
    enum_size: EnumSize,
    debug: bool
}

impl Struct
{
    fn serialize<T: ?Sized>(&mut self, key: &str, value: &T) -> Result<(), Error>
    where
        T: Serialize
    {
        self.obj.set(
            key,
            value.serialize(Serializer::new(self.enum_size, self.debug))?
        );
        Ok(())
    }
}

impl SerializeStruct for Struct
{
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T
    ) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        self.serialize(key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error>
    {
        Ok(self.obj.into())
    }
}

impl SerializeStructVariant for Struct
{
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T
    ) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        self.serialize(key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error>
    {
        Ok(self.obj.into())
    }
}

/// An implementation of a `serde` serializer for BPXSD [Value](crate::sd::Value).
pub struct Serializer
{
    enum_size: EnumSize,
    debug: bool
}

impl Serializer
{
    /// Creates a new BPXSD serializer for use with `serde`.
    ///
    /// NOTE: Only available with the `serde` cargo feature.
    ///
    /// # Arguments
    ///
    /// * `enum_size`: The size of a Rust enum.
    /// * `debug`: Whether to write debug information with BPXSD objects.
    ///
    /// returns: Serializer
    pub fn new(enum_size: EnumSize, debug: bool) -> Serializer
    {
        Serializer { enum_size, debug }
    }
}

impl serde::Serializer for Serializer
{
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = Seq;
    type SerializeTuple = Seq;
    type SerializeTupleStruct = Seq;
    type SerializeTupleVariant = Seq;
    type SerializeMap = Map;
    type SerializeStruct = Struct;
    type SerializeStructVariant = Struct;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error>
    {
        Ok(v.into())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error>
    {
        Ok(v.into())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error>
    {
        Ok(v.into())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error>
    {
        Ok(v.into())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error>
    {
        Ok(v.into())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error>
    {
        Ok(v.into())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error>
    {
        Ok(v.into())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error>
    {
        Ok(v.into())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error>
    {
        Ok(v.into())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error>
    {
        Ok(v.into())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error>
    {
        Ok(v.into())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error>
    {
        Ok(Value::from(v as u32))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error>
    {
        Ok(v.into())
    }

    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error>
    {
        Err(Error::UnsupportedType)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error>
    {
        Ok(Value::Null)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error>
    {
        Err(Error::UnsupportedType)
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error>
    {
        Err(Error::UnsupportedType)
    }

    fn serialize_unit_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str
    ) -> Result<Self::Ok, Self::Error>
    {
        Ok(match self.enum_size {
            EnumSize::U8 => Value::Uint8(variant_index as u8),
            EnumSize::U16 => Value::Uint16(variant_index as u16),
            EnumSize::U32 => Value::Uint32(variant_index)
        })
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _: &'static str,
        value: &T
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        value: &T
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize
    {
        let mut arr = Array::with_capacity(2);
        match self.enum_size {
            EnumSize::U8 => arr.as_mut().push((variant_index as u8).into()),
            EnumSize::U16 => arr.as_mut().push((variant_index as u16).into()),
            EnumSize::U32 => arr.as_mut().push(variant_index.into())
        }
        arr.as_mut().push(value.serialize(self)?);
        Ok(arr.into())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error>
    {
        Ok(Seq {
            arr: if let Some(len) = len {
                Array::with_capacity(len as _)
            } else {
                Array::new()
            },
            enum_size: self.enum_size,
            debug: self.debug
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error>
    {
        Ok(Seq {
            arr: Array::with_capacity(len as _),
            enum_size: self.enum_size,
            debug: self.debug
        })
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize
    ) -> Result<Self::SerializeTupleStruct, Self::Error>
    {
        Ok(Seq {
            arr: Array::with_capacity(len as _),
            enum_size: self.enum_size,
            debug: self.debug
        })
    }

    fn serialize_tuple_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        len: usize
    ) -> Result<Self::SerializeTupleVariant, Self::Error>
    {
        let mut arr = Array::with_capacity(len as u8 + 1);
        match self.enum_size {
            EnumSize::U8 => arr.as_mut().push((variant_index as u8).into()),
            EnumSize::U16 => arr.as_mut().push((variant_index as u16).into()),
            EnumSize::U32 => arr.as_mut().push(variant_index.into())
        }
        Ok(Seq {
            arr,
            enum_size: self.enum_size,
            debug: self.debug
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error>
    {
        Ok(Map {
            cur_obj: DebuggerOrObject::with_capacity(2, self.debug),
            arr: if let Some(len) = len {
                Array::with_capacity(len as _)
            } else {
                Array::new()
            },
            enum_size: self.enum_size,
            debug: self.debug
        })
    }

    fn serialize_struct(
        self,
        _: &'static str,
        len: usize
    ) -> Result<Self::SerializeStruct, Self::Error>
    {
        Ok(Struct {
            obj: DebuggerOrObject::with_capacity(len, self.debug),
            enum_size: self.enum_size,
            debug: self.debug
        })
    }

    fn serialize_struct_variant(
        self,
        _: &'static str,
        variant_index: u32,
        _: &'static str,
        len: usize
    ) -> Result<Self::SerializeStructVariant, Self::Error>
    {
        let mut obj = DebuggerOrObject::with_capacity(len + 1, self.debug);
        match self.enum_size {
            EnumSize::U8 => obj.set("__variant__", (variant_index as u8).into()),
            EnumSize::U16 => obj.set("__variant__", (variant_index as u16).into()),
            EnumSize::U32 => obj.set("__variant__", variant_index.into())
        }
        Ok(Struct {
            obj,
            enum_size: self.enum_size,
            debug: self.debug
        })
    }
}

#[cfg(test)]
mod tests
{
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::sd::serde::Deserializer;

    #[test]
    fn basic_enum()
    {
        #[derive(Deserialize, Serialize, Eq, PartialEq, Debug)]
        enum MyEnum
        {
            Val,
            Val1,
            Val2
        }
        let e = MyEnum::Val
            .serialize(Serializer::new(EnumSize::U8, false))
            .unwrap();
        let e1 = MyEnum::Val1
            .serialize(Serializer::new(EnumSize::U16, false))
            .unwrap();
        let e2 = MyEnum::Val2
            .serialize(Serializer::new(EnumSize::U32, false))
            .unwrap();
        assert_eq!(
            MyEnum::deserialize(Deserializer::new(EnumSize::U8, e)).unwrap(),
            MyEnum::Val
        );
        assert_eq!(
            MyEnum::deserialize(Deserializer::new(EnumSize::U16, e1)).unwrap(),
            MyEnum::Val1
        );
        assert_eq!(
            MyEnum::deserialize(Deserializer::new(EnumSize::U32, e2)).unwrap(),
            MyEnum::Val2
        );
    }

    #[test]
    fn tuple_enum()
    {
        #[derive(Deserialize, Serialize, Eq, PartialEq, Debug)]
        enum MyEnum
        {
            Val(u8),
            Val1,
            Val2(u8, u8)
        }
        let e = MyEnum::Val2(0, 42)
            .serialize(Serializer::new(EnumSize::U8, false))
            .unwrap();
        let e = MyEnum::deserialize(Deserializer::new(EnumSize::U8, e)).unwrap();
        assert_eq!(e, MyEnum::Val2(0, 42));
    }

    #[test]
    fn basic_struct()
    {
        #[derive(Deserialize, Serialize)]
        struct MyStruct
        {
            val: u8,
            val1: u8,
            val2: String,
            val3: (f32, f32, f32)
        }
        let val = MyStruct {
            val: 42,
            val1: 84,
            val2: "test string".into(),
            val3: (1.0, 2.0, 3.0)
        }
        .serialize(Serializer::new(EnumSize::U8, false))
        .unwrap();
        let test = MyStruct::deserialize(Deserializer::new(EnumSize::U8, val)).unwrap();
        assert_eq!(test.val, 42);
        assert_eq!(test.val1, 84);
        assert_eq!(test.val2, "test string");
        assert_eq!(test.val3, (1.0, 2.0, 3.0));
    }
}
