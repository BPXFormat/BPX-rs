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

use std::vec::Vec;

use crate::core::compression::Checksum;

const POLYNOMIAL: u32 = 0xEDB88320;

pub struct Crc32Checksum
{
    table: Vec<u32>,
    current: u32
}

impl Crc32Checksum
{
    pub fn new() -> Crc32Checksum
    {
        let mut table = Vec::with_capacity(256);
        for i in 0..256 {
            let mut val = i as u32;
            if (val & 0x1) != 0 {
                val = (val >> 1) ^ POLYNOMIAL;
            } else {
                val >>= 1;
            }
            table.push(val);
        }
        Crc32Checksum {
            table,
            current: 0xFFFFFFFF
        }
    }
}

impl Checksum for Crc32Checksum
{
    fn push(&mut self, buffer: &[u8])
    {
        for byte in buffer {
            let index = (self.current ^ *byte as u32) & 0xFF;
            self.current = (self.current >> 8) ^ self.table[index as usize];
        }
    }

    fn finish(mut self) -> u32
    {
        self.current ^= 0xFFFFFFFF;
        self.current
    }
}
