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

use std::io::Read;
use byteorder::ByteOrder;
use byteorder::LittleEndian;

use crate::sd::Value;
use crate::sd::Object;
use crate::sd::Array;
use crate::error::Error;
use crate::Result;

fn read_bool(stream: &mut dyn Read) -> Result<Value>
{
    let mut flag: [u8; 1] = [0; 1];

    if stream.read(&mut flag)? != 1
    {
        return Err(Error::Truncation("Read Structured Data Value (bool)"));
    }
    return Ok(Value::Bool(flag[0] == 1));
}

fn read_uint8(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 1] = [0; 1];

    if stream.read(&mut val)? != 1
    {
        return Err(Error::Truncation("Read Structured Data Value (uint8)"));
    }
    return Ok(Value::Uint8(val[0]));
}

fn read_int8(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 1] = [0; 1];

    if stream.read(&mut val)? != 1
    {
        return Err(Error::Truncation("Read Structured Data Value (int8)"));
    }
    return Ok(Value::Int8(val[0] as i8));
}

fn read_uint16(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 2] = [0; 2];

    if stream.read(&mut val)? != 2
    {
        return Err(Error::Truncation("Read Structured Data Value (uint16)"));
    }
    return Ok(Value::Uint16(LittleEndian::read_u16(&val)));
}

fn read_int16(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 2] = [0; 2];

    if stream.read(&mut val)? != 2
    {
        return Err(Error::Truncation("Read Structured Data Value (int16)"));
    }
    return Ok(Value::Int16(LittleEndian::read_i16(&val)));
}

fn read_uint32(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 4] = [0; 4];

    if stream.read(&mut val)? != 4
    {
        return Err(Error::Truncation("Read Structured Data Value (uint32)"));
    }
    return Ok(Value::Uint32(LittleEndian::read_u32(&val)));
}

fn read_int32(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 4] = [0; 4];

    if stream.read(&mut val)? != 4
    {
        return Err(Error::Truncation("Read Structured Data Value (int32)"));
    }
    return Ok(Value::Int32(LittleEndian::read_i32(&val)));
}

fn read_uint64(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 8] = [0; 8];

    if stream.read(&mut val)? != 8
    {
        return Err(Error::Truncation("Read Structured Data Value (uint64)"));
    }
    return Ok(Value::Uint64(LittleEndian::read_u64(&val)));
}

fn read_int64(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 8] = [0; 8];

    if stream.read(&mut val)? != 8
    {
        return Err(Error::Truncation("Read Structured Data Value (int64)"));
    }
    return Ok(Value::Int64(LittleEndian::read_i64(&val)));
}

fn read_float(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 4] = [0; 4];

    if stream.read(&mut val)? != 4
    {
        return Err(Error::Truncation("Read Structured Data Value (float)"));
    }
    return Ok(Value::Float(LittleEndian::read_f32(&val)));
}

fn read_double(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 8] = [0; 8];

    if stream.read(&mut val)? != 8
    {
        return Err(Error::Truncation("Read Structured Data Value (double)"));
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
            return Err(Error::Truncation("Read Structured Data Value (string)"));
        }
    }
    match String::from_utf8(curs)
    {
        Err(e) => return Err(Error::Utf8("Read Structured Data Value (string)")),
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
            return Err(Error::Truncation("Read Structured Data Value (object)"));
        }
        buf[0]
    };

    while count > 0
    {
        let mut prop: [u8; 9] = [0; 9];
        if stream.read(&mut prop)? != 9
        {
            return Err(Error::Truncation("Read Structured Data Value (object)"));
        }
        let hash = LittleEndian::read_u64(&prop[0..8]);
        let type_code = prop[8];
        match get_value_parser(type_code)
        {
            Some(func) => obj.raw_set(hash, func(stream)?),
            None => return Err(Error::Corruption(format!("Got unexpected unknown type code ({}) while reading Structured Data Object", type_code)))
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
            return Err(Error::Truncation("Read Structured Data Value (array)"));
        }
        buf[0]
    };

    while count > 0
    {
        let mut type_code: [u8; 1] = [0; 1];
        if stream.read(&mut type_code)? != 1
        {
            return Err(Error::Truncation("Read Structured Data Value (array)"));
        }
        match get_value_parser(type_code[0])
        {
            Some(func) => arr.add(func(stream)?),
            None => return Err(Error::Corruption(format!("Got unexpected unknown type code ({}) while reading Structured Data Array", type_code[0])))
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

pub fn read_structured_data(source: &mut dyn Read) -> Result<Object>
{
    return parse_object(source);
}
