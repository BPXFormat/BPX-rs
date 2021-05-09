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

use crate::header::FLAG_COMPRESS_XZ;
use crate::header::FLAG_COMPRESS_ZLIB;
use crate::header::FLAG_CHECK_CRC32;
use crate::header::FLAG_CHECK_WEAK;
use crate::header::SectionHeader;
use crate::header::MainHeader;

const COMPRESSION_THRESHOLD: u32 = 65536;

pub enum CompressionMethod
{
    Xz,
    Zlib
}

pub enum Checksum
{
    Weak,
    Crc32
}

pub struct SectionHeaderBuilder
{
    header: SectionHeader
}

impl SectionHeaderBuilder
{
    pub fn new() -> SectionHeaderBuilder
    {
        return SectionHeaderBuilder
        {
            header: SectionHeader::new()
        };
    }

    pub fn with_size(mut self, size: u32) -> Self
    {
        self.header.size = size;
        return self;
    }

    pub fn with_type(mut self, typeb: u8) -> Self
    {
        self.header.btype = typeb;
        return self;
    }

    pub fn with_compression(mut self, method: CompressionMethod) -> Self
    {
        match method
        {
            CompressionMethod::Xz => self.header.flags |= FLAG_COMPRESS_XZ,
            CompressionMethod::Zlib => self.header.flags |= FLAG_COMPRESS_ZLIB
        }
        self.header.csize = COMPRESSION_THRESHOLD;
        return self;
    }

    pub fn with_threshold(mut self, threshold: u32) -> Self
    {
        self.header.csize = threshold;
        return self;
    }

    pub fn with_checksum(mut self, chksum: Checksum) -> Self
    {
        match chksum
        {
            Checksum::Crc32 => self.header.flags |= FLAG_CHECK_CRC32,
            Checksum::Weak => self.header.flags |= FLAG_CHECK_WEAK
        }
        return self;
    }

    pub fn build(self) -> SectionHeader
    {
        return self.header;
    }
}

pub struct MainHeaderBuilder
{
    header: MainHeader
}

impl MainHeaderBuilder
{
    pub fn new() -> MainHeaderBuilder
    {
        return MainHeaderBuilder
        {
            header: MainHeader::new()
        };
    }

    pub fn with_type(mut self, typeb: u8) -> Self
    {
        self.header.btype = typeb;
        return self;
    }

    pub fn with_type_ext(mut self, type_ext: [u8; 16]) -> Self
    {
        self.header.type_ext = type_ext;
        return self;
    }

    pub fn build(self) -> MainHeader
    {
        return self.header;
    }
}
