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

//! High-level utilities to generate low-level file headers

use crate::header::{
    MainHeader,
    SectionHeader,
    FLAG_CHECK_CRC32,
    FLAG_CHECK_WEAK,
    FLAG_COMPRESS_XZ,
    FLAG_COMPRESS_ZLIB
};

const COMPRESSION_THRESHOLD: u32 = 65536;

/// The compression method to use for a section
pub enum CompressionMethod
{
    /// Use the xz compression algorithm with extreme preset
    ///
    /// *Slow but usually provides better compression*
    Xz,

    /// Use the zlib compression algorithm
    ///
    /// *Faster but does not compress as much as the xz algorithm*
    Zlib
}

/// The checksum algorithm to use for a section
pub enum Checksum
{
    /// The weak checksum is a very fast algorithm which is computed by adding all bytes of data
    ///
    /// *Not recommended for large or potentially large sections*
    Weak,

    /// Use a CRC32 algorithn to compute the checksum
    ///
    /// *This is the prefered method for all large or potentially large sections*
    Crc32
}

/// Utility to easily generate a [SectionHeader](crate::header::SectionHeader)
pub struct SectionHeaderBuilder
{
    header: SectionHeader
}

impl SectionHeaderBuilder
{
    /// Creates a new section header builder
    ///
    /// # Returns
    ///
    /// * the new section header builder
    pub fn new() -> SectionHeaderBuilder
    {
        return SectionHeaderBuilder {
            header: SectionHeader::new()
        };
    }

    /// Defines the size in bytes of the section
    ///
    /// - *By default, the size of the section is not known, and the encoder will assume a dynamic size is requested*
    ///
    /// # Arguments
    ///
    /// * `size` - the size of the section
    pub fn with_size(mut self, size: u32) -> Self
    {
        self.header.size = size;
        return self;
    }

    /// Defines the variant byte of the section
    ///
    /// - *The default value of the variant byte is 0*
    ///
    /// # Arguments
    ///
    /// * `typeb` - the variant byte of the section
    pub fn with_type(mut self, typeb: u8) -> Self
    {
        self.header.btype = typeb;
        return self;
    }

    /// Defines the compression algorithm to use when compressing the section
    ///
    /// - *The default is to not perform any compression at all*
    ///
    /// # Arguments
    ///
    /// * `method` - the [CompressionMethod](self::CompressionMethod)
    pub fn with_compression(mut self, method: CompressionMethod) -> Self
    {
        match method {
            CompressionMethod::Xz => self.header.flags |= FLAG_COMPRESS_XZ,
            CompressionMethod::Zlib => self.header.flags |= FLAG_COMPRESS_ZLIB
        }
        self.header.csize = COMPRESSION_THRESHOLD;
        return self;
    }

    /// Defines the maximum size in bytes to keep the section uncompressed
    ///
    /// - *Use a value of 0 in order to force compression all the time*
    /// - *The default threshold is set to 65536*
    ///
    /// # Arguments
    ///
    /// * `threshold` - the new threshold value
    pub fn with_threshold(mut self, threshold: u32) -> Self
    {
        self.header.csize = threshold;
        return self;
    }

    /// Defines the checksum algorithm to use when computing the checksum for the data in that section
    ///
    /// - *By default, no checksum is applied and the checksum field of the BPX Section Header is set to 0*
    ///
    /// # Arguments
    ///
    /// * `chksum` - the [Checksum](self::Checksum)
    pub fn with_checksum(mut self, chksum: Checksum) -> Self
    {
        match chksum {
            Checksum::Crc32 => self.header.flags |= FLAG_CHECK_CRC32,
            Checksum::Weak => self.header.flags |= FLAG_CHECK_WEAK
        }
        return self;
    }

    /// Consumes self and returns the generated [SectionHeader](crate::header::SectionHeader)
    ///
    /// # Returns
    ///
    /// * a new [SectionHeader](crate::header::SectionHeader)
    pub fn build(self) -> SectionHeader
    {
        return self.header;
    }
}

/// Utility to easily generate a [MainHeader](crate::header::MainHeader)
pub struct MainHeaderBuilder
{
    header: MainHeader
}

impl MainHeaderBuilder
{
    /// Creates a new main header builder
    ///
    /// # Returns
    ///
    /// * the new main header builder
    pub fn new() -> MainHeaderBuilder
    {
        return MainHeaderBuilder {
            header: MainHeader::new()
        };
    }

    /// Defines the BPX variant byte
    ///
    /// - *The default value of the variant byte is 0*
    ///
    /// # Arguments
    ///
    /// * `typeb` - the BPX variant byte
    pub fn with_type(mut self, typeb: u8) -> Self
    {
        self.header.btype = typeb;
        return self;
    }

    /// Defines the Extended Type Information field of the BPX
    ///
    /// - *The default value of the variant byte is 0*
    ///
    /// # Arguments
    ///
    /// * `type_ext` - the Extended Type Information block
    pub fn with_type_ext(mut self, type_ext: [u8; 16]) -> Self
    {
        self.header.type_ext = type_ext;
        return self;
    }

    /// Defines the version of the BPX
    ///
    /// - *The default value of the version int is given by [BPX_CURRENT_VERSION](crate::header::BPX_CURRENT_VERSION)*
    ///
    /// # Arguments
    ///
    /// * `type_ext` - the Extended Type Information block
    pub fn with_version(mut self, version: u32) -> Self
    {
        self.header.version = version;
        return self;
    }

    /// Consumes self and returns the generated [MainHeader](crate::header::MainHeader)
    ///
    /// # Returns
    ///
    /// * a new [MainHeader](crate::header::MainHeader)
    pub fn build(self) -> MainHeader
    {
        return self.header;
    }
}