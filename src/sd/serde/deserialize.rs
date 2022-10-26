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

use serde::de::{
    DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor,
};

use crate::sd::{
    serde::{EnumSize, Error},
    Array, Object, Value,
};

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Message(msg.to_string())
    }
}

struct Seq {
    enum_size: EnumSize,
    arr: Array,
}

impl<'de> SeqAccess<'de> for Seq {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(val) = self.arr.remove_at(0) {
            seed.deserialize(Deserializer::new(self.enum_size, val))
                .map(Some)
        } else {
            Ok(None)
        }
    }
}

struct Map {
    enum_size: EnumSize,
    arr: Array,
    value: Option<Object>,
}

impl<'de> MapAccess<'de> for Map {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        if let Some(obj) = self.arr.remove_at(0) {
            let obj: Object = obj.try_into()?;
            let key = obj.get("__key__").ok_or(Error::MissingMapKey)?;
            self.value = Some(obj.clone());
            seed.deserialize(Deserializer::new(self.enum_size, key.clone()))
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let obj = self.value.take().ok_or(Error::InvalidMapCall)?;
        let value = obj.get("__value__").ok_or(Error::MissingMapValue)?;
        seed.deserialize(Deserializer::new(self.enum_size, value.clone()))
    }
}

struct Struct {
    enum_size: EnumSize,
    cur_field: usize,
    fields: &'static [&'static str],
    obj: Object,
}

impl<'de> SeqAccess<'de> for Struct {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        if self.cur_field >= self.fields.len() {
            return Ok(None);
        }
        let name = self.fields[self.cur_field];
        let val = self
            .obj
            .get(name)
            .ok_or(Error::MissingStructKey(name))?
            .clone();
        let val = seed.deserialize(Deserializer::new(self.enum_size, val))?;
        self.cur_field += 1;
        Ok(Some(val))
    }
}

struct Enum {
    enum_size: EnumSize,
    val: Value,
}

impl<'de> VariantAccess<'de> for Enum {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(Error::InvalidEnum) //This case should have been catched prior to calling this, if not then BPXSD enum deserialization in serde cannot be achieved
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let mut v: Array = self.val.try_into()?;
        let val = v.remove_at(0).ok_or(Error::MissingVariantData)?;
        seed.deserialize(Deserializer::new(self.enum_size, val))
    }

    fn tuple_variant<V>(self, _: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let arr: Array = self.val.try_into()?;
        visitor.visit_seq(Seq {
            arr,
            enum_size: self.enum_size,
        })
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let obj: Object = self.val.try_into()?;
        visitor.visit_seq(Struct {
            fields,
            obj,
            cur_field: 0,
            enum_size: self.enum_size,
        })
    }
}

impl<'de> EnumAccess<'de> for Enum {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant_idx = match &mut self.val {
            Value::Array(arr) => arr.remove_at(0),
            Value::Object(obj) => obj.get("__variant__").cloned(),
            _ => None,
        }
        .ok_or(Error::InvalidEnum)?;
        let val = seed.deserialize(Deserializer::new(self.enum_size, variant_idx))?;
        Ok((val, self))
    }
}

/// An implementation of a `serde` deserializer for BPXSD [Value](crate::sd::Value).
pub struct Deserializer {
    enum_size: EnumSize,
    val: Value,
}

impl Deserializer {
    /// Creates a new BPXSD deserializer for use with `serde`.
    ///
    /// NOTE: Only available with the `serde` cargo feature.
    ///
    /// # Arguments
    ///
    /// * `enum_size`: The size of a Rust enum.
    /// * `val`: The BPXSD [Value](crate::sd::Value) to deserialize.
    ///
    /// returns: Deserializer
    pub fn new<T: Into<Value>>(enum_size: EnumSize, val: T) -> Deserializer {
        Deserializer {
            enum_size,
            val: val.into(),
        }
    }
}

impl<'de> serde::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.val {
            Value::Null => visitor.visit_none(),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::Uint8(v) => visitor.visit_u8(v),
            Value::Uint16(v) => visitor.visit_u16(v),
            Value::Uint32(v) => visitor.visit_u32(v),
            Value::Uint64(v) => visitor.visit_u64(v),
            Value::Int8(v) => visitor.visit_i8(v),
            Value::Int16(v) => visitor.visit_i16(v),
            Value::Int32(v) => visitor.visit_i32(v),
            Value::Int64(v) => visitor.visit_i64(v),
            Value::Float(v) => visitor.visit_f32(v),
            Value::Double(v) => visitor.visit_f64(v),
            Value::String(v) => visitor.visit_string(v),
            _ => Err(Error::UnsupportedType),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.val.try_into()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.val.try_into()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.val.try_into()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.val.try_into()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.val.try_into()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.val.try_into()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.val.try_into()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.val.try_into()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.val.try_into()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.val.try_into()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.val.try_into()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v: u32 = self.val.try_into()?;
        let v = char::from_u32(v).ok_or(Error::InvalidUtf32(v))?;
        visitor.visit_char(v)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let v: String = self.val.try_into()?;
        visitor.visit_str(&v)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.val.try_into()?)
    }

    fn deserialize_bytes<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_byte_buf<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.val {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_unit_struct<V>(self, _: &'static str, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Seq {
            arr: self.val.try_into()?,
            enum_size: self.enum_size,
        })
    }

    fn deserialize_tuple<V>(self, _: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _: &'static str,
        _: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(Map {
            arr: self.val.try_into()?,
            enum_size: self.enum_size,
            value: None,
        })
    }

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let obj: Object = self.val.try_into()?;
        visitor.visit_seq(Struct {
            fields,
            obj,
            cur_field: 0,
            enum_size: self.enum_size,
        })
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.enum_size {
            EnumSize::U8 => {
                match self.val {
                    Value::Uint8(v) => visitor.visit_enum((v as u32).into_deserializer()), //We got a standard C enum
                    _ => visitor.visit_enum(Enum {
                        val: self.val,
                        enum_size: self.enum_size,
                    }),
                }
            },
            EnumSize::U16 => {
                match self.val {
                    Value::Uint16(v) => visitor.visit_enum((v as u32).into_deserializer()), //We got a standard C enum
                    _ => visitor.visit_enum(Enum {
                        val: self.val,
                        enum_size: self.enum_size,
                    }),
                }
            },
            EnumSize::U32 => {
                match self.val {
                    Value::Uint32(v) => visitor.visit_enum(v.into_deserializer()), //We got a standard C enum
                    _ => visitor.visit_enum(Enum {
                        val: self.val,
                        enum_size: self.enum_size,
                    }),
                }
            },
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        //Assume the identifier is always for an enum, if not well it could throw Err or a broken value
        match self.enum_size {
            EnumSize::U8 => {
                let val: u8 = self.val.try_into()?;
                visitor.visit_u32(val as u32)
            },
            EnumSize::U16 => {
                let val: u16 = self.val.try_into()?;
                visitor.visit_u32(val as u32)
            },
            EnumSize::U32 => {
                let val: u32 = self.val.try_into()?;
                visitor.visit_u32(val)
            },
        }
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;
    use crate::sd::{Array, Object, Value};

    #[test]
    fn basic_enum() {
        #[derive(Deserialize, Eq, PartialEq, Debug)]
        enum MyEnum {
            Val,
            Val1,
            Val2,
        }
        assert_eq!(
            MyEnum::deserialize(Deserializer::new(EnumSize::U8, 0u8)).unwrap(),
            MyEnum::Val
        );
        assert_eq!(
            MyEnum::deserialize(Deserializer::new(EnumSize::U16, 1u16)).unwrap(),
            MyEnum::Val1
        );
        assert_eq!(
            MyEnum::deserialize(Deserializer::new(EnumSize::U32, 2u32)).unwrap(),
            MyEnum::Val2
        );
    }

    #[test]
    fn tuple_enum() {
        #[derive(Deserialize, Eq, PartialEq, Debug)]
        enum MyEnum {
            Val(u8),
            Val1,
            Val2(u8, u8),
        }
        let mut arr = Array::new();
        arr.add(Value::Uint32(2));
        arr.add(Value::Uint8(0));
        arr.add(Value::Uint8(42));
        let e = MyEnum::deserialize(Deserializer::new(EnumSize::U32, arr)).unwrap();
        assert_eq!(e, MyEnum::Val2(0, 42));
    }

    #[test]
    fn basic_struct() {
        #[derive(Deserialize)]
        struct MyStruct {
            val: u8,
            val1: u8,
            val2: String,
            val3: (f32, f32, f32),
        }
        let mut obj = Object::new();
        obj.set("val", 42u8.into());
        obj.set("val1", 84u8.into());
        obj.set("val2", "test string".into());
        obj.set(
            "val3",
            vec![Value::Float(1.0), Value::Float(2.0), Value::Float(3.0)].into(),
        );
        let test = MyStruct::deserialize(Deserializer::new(EnumSize::U32, obj)).unwrap();
        assert_eq!(test.val, 42);
        assert_eq!(test.val1, 84);
        assert_eq!(test.val2, "test string");
        assert_eq!(test.val3, (1.0, 2.0, 3.0));
    }
}
