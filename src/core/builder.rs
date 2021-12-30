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

//! High-level utilities to generate low-level file headers.

use crate::core::header::{
    MainHeader,
    SectionHeader,
    Struct,
    FLAG_CHECK_CRC32,
    FLAG_CHECK_WEAK,
    FLAG_COMPRESS_XZ,
    FLAG_COMPRESS_ZLIB
};

const COMPRESSION_THRESHOLD: u32 = 65536;

/// The compression method to use for a section.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CompressionMethod
{
    /// Use the xz compression algorithm with extreme preset.
    ///
    /// *Slow but usually provides better compression.*
    Xz,

    /// Use the zlib compression algorithm.
    ///
    /// *Faster but does not compress as much as the xz algorithm.*
    Zlib
}

/// The checksum algorithm to use for a section
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Checksum
{
    /// The weak checksum is a very fast algorithm which is computed
    /// by adding all bytes of data.
    ///
    /// *Not recommended for large or potentially large sections.*
    Weak,

    /// Use a CRC32 algorithn to compute the checksum.
    ///
    /// *This is the prefered method for all large or potentially
    /// large sections.*
    Crc32
}

/// Utility to easily generate a [SectionHeader](crate::header::SectionHeader).
pub struct SectionHeaderBuilder
{
    header: SectionHeader
}

impl Default for SectionHeaderBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl SectionHeaderBuilder
{
    /// Creates a new section header builder.
    pub fn new() -> SectionHeaderBuilder
    {
        SectionHeaderBuilder {
            header: SectionHeader::new()
        }
    }

    /// Defines the size in bytes of the section.
    ///
    /// *By default, the size of the section is not known, and the encoder
    /// will assume a dynamic size is requested.*
    ///
    /// # Arguments
    ///
    /// * `size`: the size in bytes of the section.
    ///
    /// returns: SectionHeaderBuilder
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::SectionHeaderBuilder;
    ///
    /// let header = SectionHeaderBuilder::new()
    ///     .size(128)
    ///     .build();
    /// assert_eq!(header.size, 128);
    /// ```
    pub fn size(&mut self, size: u32) -> &mut Self
    {
        self.header.size = size;
        self
    }

    /// Defines the type byte of the section.
    ///
    /// *The default value of the type byte is 0.*
    ///
    /// # Arguments
    ///
    /// * `ty`: the type byte of the section.
    ///
    /// returns: SectionHeaderBuilder
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::SectionHeaderBuilder;
    ///
    /// let header = SectionHeaderBuilder::new()
    ///     .ty(1)
    ///     .build();
    /// assert_eq!(header.ty, 1);
    /// ```
    pub fn ty(&mut self, ty: u8) -> &mut Self
    {
        self.header.ty = ty;
        self
    }

    /// Defines the compression algorithm to use when compressing the section.
    ///
    /// *The default is to not perform any compression at all.*
    ///
    /// # Arguments
    ///
    /// * `method`: the [CompressionMethod](self::CompressionMethod) to use for saving this section.
    ///
    /// returns: SectionHeaderBuilder
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::{CompressionMethod, SectionHeaderBuilder};
    /// use bpx::core::header::FLAG_COMPRESS_ZLIB;
    ///
    /// let header = SectionHeaderBuilder::new()
    ///     .compression(CompressionMethod::Zlib)
    ///     .build();
    /// assert_ne!(header.flags & FLAG_COMPRESS_ZLIB, 0);
    /// ```
    pub fn compression(&mut self, method: CompressionMethod) -> &mut Self
    {
        match method {
            CompressionMethod::Xz => self.header.flags |= FLAG_COMPRESS_XZ,
            CompressionMethod::Zlib => self.header.flags |= FLAG_COMPRESS_ZLIB
        }
        self.header.csize = COMPRESSION_THRESHOLD;
        self
    }

    /// Defines the maximum size in bytes to keep the section uncompressed.
    ///
    /// *Use a value of 0 in order to force compression all the time.*
    ///
    /// *The default threshold is set to 65536.*
    ///
    /// # Arguments
    ///
    /// * `threshold`: the new value of the compression threshold.
    ///
    /// returns: SectionHeaderBuilder
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::{CompressionMethod, SectionHeaderBuilder};
    ///
    /// let header = SectionHeaderBuilder::new()
    ///     .compression(CompressionMethod::Zlib)
    ///     .threshold(0)
    ///     .build();
    /// // The compression threshold value is stored in csize
    /// assert_eq!(header.csize, 0);
    /// ```
    pub fn threshold(&mut self, threshold: u32) -> &mut Self
    {
        self.header.csize = threshold;
        self
    }

    /// Defines the checksum algorithm to use when computing
    /// the checksum for the data in that section.
    ///
    /// *By default, no checksum is applied and the checksum
    /// field of the BPX Section Header is set to 0.*
    ///
    /// # Arguments
    ///
    /// * `chksum`: the new [Checksum](self::Checksum) algorithm to use for data verification.
    ///
    /// returns: SectionHeaderBuilder
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::{Checksum, SectionHeaderBuilder};
    /// use bpx::core::header::FLAG_CHECK_CRC32;
    ///
    /// let header = SectionHeaderBuilder::new()
    ///     .checksum(Checksum::Crc32)
    ///     .build();
    /// assert_ne!(header.flags & FLAG_CHECK_CRC32, 0);
    /// ```
    pub fn checksum(&mut self, chksum: Checksum) -> &mut Self
    {
        match chksum {
            Checksum::Crc32 => self.header.flags |= FLAG_CHECK_CRC32,
            Checksum::Weak => self.header.flags |= FLAG_CHECK_WEAK
        }
        self
    }

    /// Returns the generated [SectionHeader](crate::header::SectionHeader).
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::{Checksum, CompressionMethod, SectionHeaderBuilder};
    /// use bpx::core::header::{FLAG_CHECK_CRC32, FLAG_COMPRESS_ZLIB};
    ///
    /// let header = SectionHeaderBuilder::new()
    ///     .size(128)
    ///     .ty(1)
    ///     .compression(CompressionMethod::Zlib)
    ///     .threshold(0)
    ///     .checksum(Checksum::Crc32)
    ///     .build();
    /// assert_eq!(header.size, 128);
    /// assert_eq!(header.ty, 1);
    /// assert_ne!(header.flags & FLAG_COMPRESS_ZLIB, 0);
    /// assert_eq!(header.csize, 0);
    /// assert_ne!(header.flags & FLAG_CHECK_CRC32, 0);
    /// ```
    pub fn build(&self) -> SectionHeader
    {
        self.header
    }
}

/// Utility to easily generate a [MainHeader](crate::header::MainHeader).
pub struct MainHeaderBuilder
{
    header: MainHeader
}

impl Default for MainHeaderBuilder
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl MainHeaderBuilder
{
    /// Creates a new main header builder.
    pub fn new() -> MainHeaderBuilder
    {
        MainHeaderBuilder {
            header: MainHeader::new()
        }
    }

    /// Defines the BPX type byte.
    ///
    /// *The default value of the type byte is 0.*
    ///
    /// # Arguments
    ///
    /// * `ty`: the BPX type byte.
    ///
    /// returns: MainHeaderBuilder
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::MainHeaderBuilder;
    ///
    /// let header = MainHeaderBuilder::new()
    ///     .ty('M' as u8)
    ///     .build();
    /// assert_eq!(header.ty, 'M' as u8);
    /// ```
    pub fn ty(&mut self, ty: u8) -> &mut Self
    {
        self.header.ty = ty;
        self
    }

    /// Defines the Extended Type Information field of the BPX.
    ///
    /// *By default Extended Type Information is filled with zeros.*
    ///
    /// # Arguments
    ///
    /// * `type_ext`: the Extended Type Information block.
    ///
    /// returns: MainHeaderBuilder
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::MainHeaderBuilder;
    ///
    /// let header = MainHeaderBuilder::new()
    ///     .type_ext([1; 16])
    ///     .build();
    /// assert_eq!(header.type_ext, [1; 16]);
    /// ```
    pub fn type_ext(&mut self, type_ext: [u8; 16]) -> &mut Self
    {
        self.header.type_ext = type_ext;
        self
    }

    /// Defines the version of the BPX.
    ///
    /// *The default value of the version int is given by
    /// [BPX_CURRENT_VERSION](crate::header::BPX_CURRENT_VERSION).*
    ///
    /// **Note: A version which is not specified in [KNOWN_VERSIONS](crate::header::KNOWN_VERSIONS)
    /// will cause the decoder to fail loading the file, complaining that
    /// the file is corrupted.**
    ///
    /// # Arguments
    ///
    /// * `version`: the new version of the BPX.
    ///
    /// returns: MainHeaderBuilder
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::MainHeaderBuilder;
    ///
    /// let header = MainHeaderBuilder::new()
    ///     .version(1)
    ///     .build();
    /// assert_eq!(header.version, 1);
    /// ```
    pub fn version(&mut self, version: u32) -> &mut Self
    {
        self.header.version = version;
        self
    }

    /// Returns the generated [MainHeader](crate::header::MainHeader).
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::builder::MainHeaderBuilder;
    ///
    /// let header = MainHeaderBuilder::new()
    ///     .ty('M' as u8)
    ///     .type_ext([1; 16])
    ///     .version(1)
    ///     .build();
    /// assert_eq!(header.ty, 'M' as u8);
    /// assert_eq!(header.type_ext, [1; 16]);
    /// assert_eq!(header.version, 1);
    /// ```
    pub fn build(&self) -> MainHeader
    {
        self.header
    }
}

impl From<&mut MainHeaderBuilder> for MainHeader
{
    fn from(builder: &mut MainHeaderBuilder) -> Self
    {
        builder.build()
    }
}

impl From<&mut SectionHeaderBuilder> for SectionHeader
{
    fn from(builder: &mut SectionHeaderBuilder) -> Self
    {
        builder.build()
    }
}

impl From<MainHeaderBuilder> for MainHeader
{
    fn from(builder: MainHeaderBuilder) -> Self
    {
        builder.build()
    }
}

impl From<SectionHeaderBuilder> for SectionHeader
{
    fn from(builder: SectionHeaderBuilder) -> Self
    {
        builder.build()
    }
}
