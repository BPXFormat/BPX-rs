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
    enum_size: EnumSize
}

impl Seq
{
    fn serialize<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize
    {
        self.arr
            .add(value.serialize(Serializer::new(self.enum_size))?);
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
    cur_obj: Object,
    arr: Array,
    enum_size: EnumSize
}

impl Map
{
    fn check_update(&mut self)
    {
        if self.cur_obj.get("__key__").is_some() && self.cur_obj.get("__value__").is_some() {
            let val = std::mem::replace(&mut self.cur_obj, Object::with_capacity(2));
            self.arr.add(val.into());
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
        self.cur_obj
            .set("__key__", key.serialize(Serializer::new(self.enum_size))?);
        self.check_update();
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize
    {
        self.cur_obj.set(
            "__value__",
            value.serialize(Serializer::new(self.enum_size))?
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
    obj: Object,
    enum_size: EnumSize
}

impl Struct
{
    fn serialize<T: ?Sized>(&mut self, key: &str, value: &T) -> Result<(), Error>
    where
        T: Serialize
    {
        self.obj
            .set(key, value.serialize(Serializer::new(self.enum_size))?);
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

pub struct Serializer
{
    enum_size: EnumSize
}

impl Serializer
{
    pub fn new(enum_size: EnumSize) -> Serializer
    {
        Serializer { enum_size }
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
            EnumSize::U8 => arr.add((variant_index as u8).into()),
            EnumSize::U16 => arr.add((variant_index as u16).into()),
            EnumSize::U32 => arr.add(variant_index.into())
        }
        arr.add(value.serialize(self)?);
        Ok(arr.into())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error>
    {
        Ok(Seq {
            arr: if let Some(len) = len {
                Array::with_capacity(len)
            } else {
                Array::new()
            },
            enum_size: self.enum_size
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error>
    {
        Ok(Seq {
            arr: Array::with_capacity(len),
            enum_size: self.enum_size
        })
    }

    fn serialize_tuple_struct(
        self,
        _: &'static str,
        len: usize
    ) -> Result<Self::SerializeTupleStruct, Self::Error>
    {
        Ok(Seq {
            arr: Array::with_capacity(len),
            enum_size: self.enum_size
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
        let mut arr = Array::with_capacity(len + 1);
        match self.enum_size {
            EnumSize::U8 => arr.add((variant_index as u8).into()),
            EnumSize::U16 => arr.add((variant_index as u16).into()),
            EnumSize::U32 => arr.add(variant_index.into())
        }
        Ok(Seq {
            arr,
            enum_size: self.enum_size
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error>
    {
        Ok(Map {
            cur_obj: Object::with_capacity(2),
            arr: if let Some(len) = len {
                Array::with_capacity(len)
            } else {
                Array::new()
            },
            enum_size: self.enum_size
        })
    }

    fn serialize_struct(
        self,
        _: &'static str,
        len: usize
    ) -> Result<Self::SerializeStruct, Self::Error>
    {
        Ok(Struct {
            obj: Object::with_capacity(len),
            enum_size: self.enum_size
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
        let mut obj = Object::with_capacity(len + 1);
        match self.enum_size {
            EnumSize::U8 => obj.set("__variant__", (variant_index as u8).into()),
            EnumSize::U16 => obj.set("__variant__", (variant_index as u16).into()),
            EnumSize::U32 => obj.set("__variant__", variant_index.into())
        }
        Ok(Struct {
            obj,
            enum_size: self.enum_size
        })
    }
}
