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

// The BPX Structured Data format (BPXSD)

use std::collections::HashMap;
use std::collections::hash_map::Keys;
use std::vec::Vec;
use std::string::String;
use std::ops::Index;
use std::ops::IndexMut;
use std::io::Read;
use std::io::Write;
use std::io::Result;
use std::io::Error;
use std::io::ErrorKind;
use byteorder::ByteOrder;
use byteorder::LittleEndian;

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

#[derive(PartialEq, Clone)]
pub struct Array
{
    data: Vec<Value>
}

impl Array
{
    pub fn new() -> Array
    {
        return Array
        {
            data: Vec::new()
        }
    }

    pub fn add(&mut self, v: Value)
    {
        self.data.push(v);
    }

    pub fn remove_at(&mut self, pos: usize)
    {
        self.data.remove(pos);
    }

    pub fn remove(&mut self, item: Value)
    {
        for i in 0..self.data.len()
        {
            if self.data[i] == item
            {
                self.data.remove(i);
            }
        }
    }

    pub fn get(&self, pos: usize) -> Option<&Value>
    {
        return self.data.get(pos);
    }

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
        self.raw_set(super::utils::hash(name), value);
        self.prop_names.add(Value::String(String::from(name)));
    }

    pub fn raw_get(&self, hash: u64) -> Option<&Value>
    {
        return self.props.get(&hash);
    }

    pub fn get(&self, name: &str) -> Option<&Value>
    {
        return self.raw_get(super::utils::hash(name));
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
}

impl Index<&str> for Object
{
    type Output = Value;

    fn index<'a>(&'a self, name: &str) -> &'a Value
    {
        return &self.props.index(&super::utils::hash(name));
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

fn read_bool(stream: &mut dyn Read) -> Result<Value>
{
    let mut flag: [u8; 1] = [0; 1];

    if stream.read(&mut flag)? != 1
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Bool(flag[0] == 1));
}

fn read_uint8(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 1] = [0; 1];

    if stream.read(&mut val)? != 1
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Uint8(val[0]));
}

fn read_int8(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 1] = [0; 1];

    if stream.read(&mut val)? != 1
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Int8(val[0] as i8));
}

fn read_uint16(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 2] = [0; 2];

    if stream.read(&mut val)? != 2
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Uint16(LittleEndian::read_u16(&val)));
}

fn read_int16(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 2] = [0; 2];

    if stream.read(&mut val)? != 2
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Int16(LittleEndian::read_i16(&val)));
}

fn read_uint32(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 4] = [0; 4];

    if stream.read(&mut val)? != 4
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Uint32(LittleEndian::read_u32(&val)));
}

fn read_int32(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 4] = [0; 4];

    if stream.read(&mut val)? != 4
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Int32(LittleEndian::read_i32(&val)));
}

fn read_uint64(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 8] = [0; 8];

    if stream.read(&mut val)? != 8
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Uint64(LittleEndian::read_u64(&val)));
}

fn read_int64(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 8] = [0; 8];

    if stream.read(&mut val)? != 8
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Int64(LittleEndian::read_i64(&val)));
}

fn read_float(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 4] = [0; 4];

    if stream.read(&mut val)? != 4
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Float(LittleEndian::read_f32(&val)));
}

fn read_double(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 8] = [0; 8];

    if stream.read(&mut val)? != 8
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Double(LittleEndian::read_f64(&val)));
}

fn read_string(stream: &mut dyn Read) -> Result<Value>
{
    let mut curs: Vec<u8> = Vec::new();
    let mut chr: [u8; 1] = [0; 1]; //read char by char with a buffer

    stream.read(&mut chr)?;
    while chr[0] != 0x0
    {
        curs.push(chr[0]);
        let res = stream.read(&mut chr)?;
        if res != 1
        {
            return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
        }
    }
    match String::from_utf8(curs)
    {
        Err(e) => return Err(Error::new(ErrorKind::InvalidData, format!("[BPX] error loading utf8 string: {}", e))),
        Ok(v) => return Ok(Value::String(v))
    }
}

fn parse_object(stream: &mut dyn Read) -> Result<Object>
{
    let mut obj = Object::new();
    let mut count =
    {
        let mut buf: [u8; 1] = [0; 1];
        if stream.read(&mut buf)? != 1
        {
            return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Object"));
        }
        buf[0]
    };

    while count > 0
    {
        let mut prop: [u8; 9] = [0; 9];
        if stream.read(&mut prop)? != 9
        {
            return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Object"));
        }
        let hash = LittleEndian::read_u64(&prop[0..8]);
        let type_code = prop[8];
        match get_value_parser(type_code)
        {
            Some(func) => obj.raw_set(hash, func(stream)?),
            None => return Err(Error::new(ErrorKind::InvalidData, format!("[BPX] Got unexpected unknown type code ({}) from Structured Data Object", type_code)))
        }
        count -= 1;
    }
    return Ok(obj);
}

fn parse_array(stream: &mut dyn Read) -> Result<Array>
{
    let mut arr = Array::new();
    let mut count =
    {
        let mut buf: [u8; 1] = [0; 1];
        if stream.read(&mut buf)? != 1
        {
            return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Object"));
        }
        buf[0]
    };

    while count > 0
    {
        let mut type_code: [u8; 1] = [0; 1];
        if stream.read(&mut type_code)? != 1
        {
            return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Object"));
        }
        match get_value_parser(type_code[0])
        {
            Some(func) => arr.add(func(stream)?),
            None => return Err(Error::new(ErrorKind::InvalidData, format!("[BPX] Got unexpected unknown type code ({}) from Structured Data Object", type_code[0])))
        }
        count -= 1;
    }
    return Ok(arr);
}

fn get_value_parser(type_code: u8) -> Option<fn (stream: &mut dyn Read) -> Result<Value>>
{
    match type_code
    {
        0x0 => Some(|_| { return Ok(Value::Null); }),
        0x1 => Some(read_bool),
        0x2 => Some(read_uint8),
        0x3 => Some(read_uint16),
        0x4 => Some(read_uint32),
        0x5 => Some(read_uint64),
        0x6 => Some(read_int8),
        0x7 => Some(read_int16),
        0x8 => Some(read_int32),
        0x9 => Some(read_int64),
        0xA => Some(read_float),
        0xB => Some(read_double),
        0xC => Some(read_string),
        0xD => Some(|stream| { return Ok(Value::Array(parse_array(stream)?)); }),
        0xE => Some(|stream| { return Ok(Value::Object(parse_object(stream)?)); }),
        _ => None
    }
}

fn get_value_type_code(val: &Value) -> u8
{
    match val
    {
        Value::Null => 0x0,
        Value::Bool(_) => 0x1,
        Value::Uint8(_) => 0x2,
        Value::Uint16(_) => 0x3,
        Value::Uint32(_) => 0x4,
        Value::Uint64(_) => 0x5,
        Value::Int8(_) => 0x6,
        Value::Int16(_) => 0x7,
        Value::Int32(_) => 0x8,
        Value::Int64(_) => 0x9,
        Value::Float(_) => 0xA,
        Value::Double(_) => 0xB,
        Value::String(_) => 0xC,
        Value::Array(_) => 0xD,
        Value::Object(_) => 0xE
    }
}

fn write_value(val: &Value) -> Result<Vec<u8>>
{
    let mut buf = Vec::new();

    match val
    {
        Value::Null => (),
        Value::Bool(b) =>
        {
            if *b
            {
                buf.push(1);
            }
            else
            {
                buf.push(0);
            }
        },
        Value::Uint8(v) => buf.push(*v),
        Value::Uint16(v) =>
        {
            let mut b: [u8; 2] = [0; 2];
            LittleEndian::write_u16(&mut b, *v);
            buf.extend_from_slice(&b);
        },
        Value::Uint32(v) =>
        {
            let mut b: [u8; 4] = [0; 4];
            LittleEndian::write_u32(&mut b, *v);
            buf.extend_from_slice(&b);
        },
        Value::Uint64(v) =>
        {
            let mut b: [u8; 8] = [0; 8];
            LittleEndian::write_u64(&mut b, *v);
            buf.extend_from_slice(&b);
        },
        Value::Int8(v) => buf.push(*v as u8),
        Value::Int16(v) =>
        {
            let mut b: [u8; 2] = [0; 2];
            LittleEndian::write_i16(&mut b, *v);
            buf.extend_from_slice(&b);
        },
        Value::Int32(v) =>
        {
            let mut b: [u8; 4] = [0; 4];
            LittleEndian::write_i32(&mut b, *v);
            buf.extend_from_slice(&b);
        },
        Value::Int64(v) =>
        {
            let mut b: [u8; 8] = [0; 8];
            LittleEndian::write_i64(&mut b, *v);
            buf.extend_from_slice(&b);
        },
        Value::Float(v) =>
        {
            let mut b: [u8; 4] = [0; 4];
            LittleEndian::write_f32(&mut b, *v);
            buf.extend_from_slice(&b);
        },
        Value::Double(v) =>
        {
            let mut b: [u8; 8] = [0; 8];
            LittleEndian::write_f64(&mut b, *v);
            buf.extend_from_slice(&b);
        },
        Value::String(s) =>
        {
            buf.extend_from_slice(s.as_bytes());
            buf.push(0x0); //Add null byte terminator
        },
        Value::Array(arr) => buf.append(&mut write_array(arr)?),
        Value::Object(obj) => buf.append(&mut write_object(obj)?)
    }
    return Ok(buf);
}

fn write_object(obj: &Object) -> Result<Vec<u8>>
{
    let mut v: Vec<u8> = Vec::new();
    let count = obj.prop_count();

    if count > 255
    {
        return Err(Error::new(ErrorKind::InvalidInput, format!("[BPX] Structured Data only supports up to 255 maximum values in either array or object, got {} values", count)));
    }
    v.push(count as u8);
    for hash in obj.get_keys()
    {
        let val = &obj[*hash];
        let mut head: [u8; 9] = [0; 9];
        LittleEndian::write_u64(&mut head[0..8], *hash);
        head[8] = get_value_type_code(val);
        v.extend_from_slice(&head);
        v.append(&mut write_value(val)?);
    }
    return Ok(v);
}

fn write_array(arr: &Array) -> Result<Vec<u8>>
{
    let mut v: Vec<u8> = Vec::new();
    let count = arr.len();

    if count > 255
    {
        return Err(Error::new(ErrorKind::InvalidInput, format!("[BPX] Structured Data only supports up to 255 maximum values in either array or object, got {} values", count)));
    }
    v.push(count as u8);
    for i in 0..count
    {
        let val = &arr[i];
        v.push(get_value_type_code(val));
        v.append(&mut write_value(val)?);
    }
    return Ok(v);
}

pub fn load_structured_data(source: &mut dyn Read) -> Result<Object>
{
    return parse_object(source);
}

pub fn write_structured_data(dest: &mut dyn Write, obj: &Object) -> Result<()>
{
    let bytes = write_object(obj)?;
    dest.write(&bytes)?;
    return Ok(());
}

pub struct DebugSymbols
{
    symbols: HashMap<u64, String>
}

impl DebugSymbols
{
    pub fn load(obj: &Object) -> Result<DebugSymbols>
    {
        let mut symbols = HashMap::new();

        if let Some(val) = obj.get("__debug__")
        {
            match val
            {
                Value::Array(arr) =>
                {
                    for i in 0..arr.len()
                    {
                        match &arr[i]
                        {
                            Value::String(s) =>
                            {
                                symbols.insert(super::utils::hash(&s), s.clone());
                            }
                            _ => return Err(Error::new(ErrorKind::InvalidData, "[BPX] Wrong value type for debugging symbols"))
                        }
                    }
                },
                _ => return Err(Error::new(ErrorKind::InvalidData, "[BPX] Wrong value type for debugging symbols"))
            }
        }
        return Ok(DebugSymbols
        {
            symbols: symbols
        });
    }

    pub fn lookup(&self, hash: u64) -> String
    {
        match self.symbols.get(&hash)
        {
            Some(s) => return s.clone(), //We have a debug symbol for the current property
            None => format!("{:#X}", hash) //We don't, return hash value as hex string
        }
    }
}