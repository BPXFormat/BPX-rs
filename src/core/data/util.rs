// Copyright (c) 2022, BlockProject 3D
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

use std::io::{Read, Result, Seek, SeekFrom, Write};

const READ_BLOCK_SIZE: usize = 8192;
#[cfg(not(test))]
const SHIFT_BUF_SIZE: usize = 8192;
#[cfg(test)]
const SHIFT_BUF_SIZE: usize = 4;

pub struct IoReadBuffer {
    buffer: [u8; READ_BLOCK_SIZE],
    length: usize,
    cursor: usize,
}

impl IoReadBuffer {
    pub fn new() -> IoReadBuffer {
        IoReadBuffer {
            buffer: [0; READ_BLOCK_SIZE],
            length: 0,
            cursor: 0,
        }
    }

    pub fn read<F: FnMut(&mut [u8]) -> Result<usize>>(
        &mut self,
        data: &mut [u8],
        mut read_block: F,
    ) -> Result<usize> {
        let mut cnt: usize = 0;

        for byte in data {
            if self.cursor >= self.length {
                self.cursor = 0;
                self.length = read_block(&mut self.buffer)?;
            }
            if self.cursor < self.length {
                *byte = self.buffer[self.cursor];
                self.cursor += 1;
                cnt += 1;
            }
        }
        Ok(cnt)
    }

    pub fn flush(&mut self) {
        self.cursor = 0;
        self.length = 0;
    }

    pub fn inverted_position(&self) -> u64 {
        self.length as u64 - self.cursor as u64
    }
}

pub fn shift_left<T: Read + Write + Seek>(mut data: T, mut length: u32) -> Result<()> {
    let cursor = data.stream_position()?;
    let mut buf = [0; SHIFT_BUF_SIZE];
    if length > cursor as u32 {
        length = cursor as u32;
    }
    let mut destination = cursor - length as u64;
    let mut source = cursor;
    loop {
        data.seek(SeekFrom::Start(source))?;
        //The reason why this is not a read_fill is because the section data buffer is designed to
        // always read as much as possible into the input buffer.
        let len = data.read(&mut buf)?;
        if len == 0 {
            break;
        }
        source += len as u64;
        data.seek(SeekFrom::Start(destination))?;
        data.write_all(&buf[..len])?;
        destination += len as u64;
    }
    data.seek(SeekFrom::Start(cursor))?;
    Ok(())
}

pub fn shift_right<T: Read + Write + Seek>(mut data: T, size: u64, length: u32) -> Result<()> {
    let cursor = data.stream_position()?;
    let mut buf = [0; SHIFT_BUF_SIZE];
    let mut source = size;
    let mut destination = source + length as u64;
    while source > cursor {
        let nextsize = std::cmp::min(SHIFT_BUF_SIZE as u64, source - cursor);
        data.seek(SeekFrom::Start(source - nextsize))?;
        data.read_exact(&mut buf[..nextsize as usize])?;
        data.seek(SeekFrom::Start(destination - nextsize))?;
        data.write_all(&buf[..nextsize as usize])?;
        source -= nextsize;
        destination -= nextsize;
    }
    data.seek(SeekFrom::Start(cursor))?;
    Ok(())
}
