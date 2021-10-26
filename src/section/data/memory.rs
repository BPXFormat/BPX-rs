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

use std::io::{Read, Result, Seek, SeekFrom, Write};

use bpx::section::SectionData;

pub struct InMemorySection
{
    data: Vec<u8>,
    cursor: usize,
    cur_size: usize
}

impl InMemorySection
{
    pub fn new(data: Vec<u8>) -> InMemorySection
    {
        return InMemorySection {
            data: data,
            cursor: 0,
            cur_size: 0
        };
    }
}

impl Read for InMemorySection
{
    fn read(&mut self, data: &mut [u8]) -> Result<usize>
    {
        for i in 0..data.len() {
            if self.cursor >= self.data.len() {
                return Ok(i);
            }
            data[i] = self.data[self.cursor];
            self.cursor += 1;
        }
        return Ok(data.len());
    }
}

impl Write for InMemorySection
{
    fn write(&mut self, data: &[u8]) -> Result<usize>
    {
        for i in 0..data.len() {
            if self.cursor >= self.data.len() {
                return Ok(i);
            }
            self.data[self.cursor] = data[i];
            self.cursor += 1;
            if self.cursor >= self.cur_size {
                self.cur_size += 1
            }
        }
        return Ok(data.len());
    }

    fn flush(&mut self) -> Result<()>
    {
        return Ok(());
    }
}

impl Seek for InMemorySection
{
    fn seek(&mut self, state: SeekFrom) -> Result<u64>
    {
        match state {
            SeekFrom::Start(pos) => self.cursor = pos as usize,
            SeekFrom::End(pos) => self.cursor = self.cursor.wrapping_add(pos as usize),
            SeekFrom::Current(pos) => self.cursor = self.cursor.wrapping_add(pos as usize)
        }
        return Ok(self.cursor as u64);
    }
}

impl SectionData for InMemorySection
{
    fn load_in_memory(&mut self) -> Result<Vec<u8>>
    {
        return Ok(self.data.clone());
    }

    fn size(&self) -> usize
    {
        return self.cur_size;
    }
}
