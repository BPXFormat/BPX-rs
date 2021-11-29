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
    fs::File,
    io::{Read, Result, Seek, SeekFrom, Write}
};
use crate::core::SectionData;

const READ_BLOCK_SIZE: usize = 8192;

pub struct FileBasedSection
{
    data: File,
    buffer: [u8; READ_BLOCK_SIZE],
    written: usize,
    cursor: usize,
    cur_size: usize,
    seek_ptr: u64
}

impl FileBasedSection
{
    pub fn new(data: File) -> FileBasedSection
    {
        FileBasedSection {
            data,
            buffer: [0; READ_BLOCK_SIZE],
            written: 0,
            cursor: usize::MAX,
            cur_size: 0,
            seek_ptr: 0
        }
    }
}

impl Read for FileBasedSection
{
    fn read(&mut self, data: &mut [u8]) -> Result<usize>
    {
        let mut cnt: usize = 0;

        for byte in data {
            if self.cursor >= self.written {
                self.cursor = 0;
                self.written = self.data.read(&mut self.buffer)?;
            }
            if self.cursor < self.written {
                *byte = self.buffer[self.cursor];
                self.cursor += 1;
                cnt += 1;
            }
        }
        Ok(cnt)
    }
}

impl Write for FileBasedSection
{
    fn write(&mut self, data: &[u8]) -> Result<usize>
    {
        let len = self.data.write(data)?;
        if self.seek_ptr >= self.cur_size as u64 {
            self.cur_size += len;
            self.seek_ptr += len as u64;
        }
        Ok(len)
    }

    fn flush(&mut self) -> Result<()>
    {
        self.data.seek(SeekFrom::Current(self.cursor as i64))?;
        self.cursor = usize::MAX;
        self.data.flush()
    }
}

impl Seek for FileBasedSection
{
    fn seek(&mut self, state: SeekFrom) -> Result<u64>
    {
        self.seek_ptr = self.data.seek(state)?;
        Ok(self.seek_ptr)
    }
}

impl SectionData for FileBasedSection
{
    fn size(&self) -> usize
    {
        self.cur_size
    }
}
