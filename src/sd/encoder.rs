// Copyright (c) 2023, BlockProject 3D
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

use std::io::Write;

use bytesutil::WriteBytes;

use crate::sd::{error::Error, Array, Object, Result, Value};

fn get_value_type_code(val: &Value) -> u8 {
    match val {
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
        Value::Object(_) => 0xE,
    }
}

fn write_value(val: &Value, max_depth: &mut usize) -> Result<Vec<u8>> {
    let mut buf = Vec::new();

    match val {
        Value::Null => (),
        Value::Bool(b) => {
            if *b {
                buf.push(1);
            } else {
                buf.push(0);
            }
        },
        Value::Uint8(v) => buf.push(*v),
        Value::Uint16(v) => {
            let mut b: [u8; 2] = [0; 2];
            (*v).write_bytes_le(&mut b);
            buf.extend_from_slice(&b);
        },
        Value::Uint32(v) => {
            let mut b: [u8; 4] = [0; 4];
            (*v).write_bytes_le(&mut b);
            buf.extend_from_slice(&b);
        },
        Value::Uint64(v) => {
            let mut b: [u8; 8] = [0; 8];
            (*v).write_bytes_le(&mut b);
            buf.extend_from_slice(&b);
        },
        Value::Int8(v) => buf.push(*v as u8),
        Value::Int16(v) => {
            let mut b: [u8; 2] = [0; 2];
            (*v).write_bytes_le(&mut b);
            buf.extend_from_slice(&b);
        },
        Value::Int32(v) => {
            let mut b: [u8; 4] = [0; 4];
            (*v).write_bytes_le(&mut b);
            buf.extend_from_slice(&b);
        },
        Value::Int64(v) => {
            let mut b: [u8; 8] = [0; 8];
            (*v).write_bytes_le(&mut b);
            buf.extend_from_slice(&b);
        },
        Value::Float(v) => {
            let mut b: [u8; 4] = [0; 4];
            (*v).write_bytes_le(&mut b);
            buf.extend_from_slice(&b);
        },
        Value::Double(v) => {
            let mut b: [u8; 8] = [0; 8];
            (*v).write_bytes_le(&mut b);
            buf.extend_from_slice(&b);
        },
        Value::String(s) => {
            buf.extend_from_slice(s.as_bytes());
            buf.push(0x0); //Add null byte terminator
        },
        Value::Array(arr) => buf.append(&mut write_array(arr, max_depth)?),
        Value::Object(obj) => buf.append(&mut write_object(obj, max_depth)?),
    }
    Ok(buf)
}

fn write_object(obj: &Object, max_depth: &mut usize) -> Result<Vec<u8>> {
    let mut v: Vec<u8> = Vec::new();
    let count = obj.len();

    *max_depth -= 1;
    if *max_depth == 0 {
        return Err(Error::MaxDepthExceeded);
    }
    if count > 255 {
        return Err(Error::CapacityExceeded(count));
    }
    v.push(count as u8);
    for (hash, val) in obj {
        let mut head: [u8; 9] = [0; 9];
        hash.into_inner().write_bytes_le(&mut head[0..8]);
        head[8] = get_value_type_code(val);
        v.extend_from_slice(&head);
        v.append(&mut write_value(val, max_depth)?);
    }
    Ok(v)
}

fn write_array(arr: &Array, max_depth: &mut usize) -> Result<Vec<u8>> {
    let mut v: Vec<u8> = Vec::new();
    let count = arr.len();

    *max_depth -= 1;
    if *max_depth == 0 {
        return Err(Error::MaxDepthExceeded);
    }
    if count > 255 {
        return Err(Error::CapacityExceeded(count));
    }
    v.push(count as u8);
    for i in 0..count {
        let val = &arr[i];
        v.push(get_value_type_code(val));
        v.append(&mut write_value(val, max_depth)?);
    }
    Ok(v)
}

pub fn write_structured_data<TWrite: Write>(
    mut dest: TWrite,
    obj: &Object,
    mut max_depth: usize,
) -> Result<()> {
    let bytes = write_object(obj, &mut max_depth)?;
    dest.write_all(&bytes)?;
    Ok(())
}
