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

//! High-level utilities to generate low-level file headers.

use bytesutil::StaticByteBuf;

use crate::core::header::{
    MainHeader, SectionHeader, Struct, FLAG_CHECK_CRC32, FLAG_CHECK_WEAK, FLAG_COMPRESS_XZ,
    FLAG_COMPRESS_ZLIB,
};

use super::{DEFAULT_COMPRESSION_THRESHOLD, DEFAULT_MEMORY_THRESHOLD};

/// The compression method to use for a section.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CompressionMethod {
    /// Use the xz compression algorithm with extreme preset.
    ///
    /// *Slow but usually provides better compression.*
    Xz,

    /// Use the zlib compression algorithm.
    ///
    /// *Faster but does not compress as much as the xz algorithm.*
    Zlib,
}

/// The checksum algorithm to use for a section
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Checksum {
    /// The weak checksum is a very fast algorithm which is computed
    /// by adding all bytes of data.
    ///
    /// *Not recommended for large or potentially large sections.*
    Weak,

    /// Use a CRC32 algorithm to compute the checksum.
    ///
    /// *This is the preferred method for all large or potentially
    /// large sections.*
    Crc32,
}

/// Utility to easily generate a [SectionHeader](SectionHeader).
pub struct SectionOptions {
    header: SectionHeader,
}

impl Default for SectionOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl SectionOptions {
    /// Creates a new set of options for a BPX section.
    pub fn new() -> SectionOptions {
        SectionOptions {
            header: SectionHeader::new(),
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
    /// returns: SectionOptions
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::SectionOptions;
    ///
    /// let header = SectionOptions::new()
    ///     .size(128)
    ///     .build();
    /// assert_eq!(header.size, 128);
    /// ```
    pub fn size(&mut self, size: u32) -> &mut Self {
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
    /// returns: SectionOptions
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::SectionOptions;
    ///
    /// let header = SectionOptions::new()
    ///     .ty(1)
    ///     .build();
    /// assert_eq!(header.ty, 1);
    /// ```
    pub fn ty(&mut self, ty: u8) -> &mut Self {
        self.header.ty = ty;
        self
    }

    /// Defines the compression algorithm to use when compressing the section.
    ///
    /// *The default is to not perform any compression at all.*
    ///
    /// # Arguments
    ///
    /// * `method`: the [CompressionMethod](CompressionMethod) to use for saving this section.
    ///
    /// returns: SectionOptions
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::{CompressionMethod, SectionOptions};
    /// use bpx::core::header::FLAG_COMPRESS_ZLIB;
    ///
    /// let header = SectionOptions::new()
    ///     .compression(CompressionMethod::Zlib)
    ///     .build();
    /// assert_ne!(header.flags & FLAG_COMPRESS_ZLIB, 0);
    /// ```
    pub fn compression(&mut self, method: CompressionMethod) -> &mut Self {
        match method {
            CompressionMethod::Xz => self.header.flags |= FLAG_COMPRESS_XZ,
            CompressionMethod::Zlib => self.header.flags |= FLAG_COMPRESS_ZLIB,
        }
        self.header.csize = DEFAULT_COMPRESSION_THRESHOLD;
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
    /// returns: SectionOptions
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::{CompressionMethod, SectionOptions};
    ///
    /// let header = SectionOptions::new()
    ///     .compression(CompressionMethod::Zlib)
    ///     .threshold(0)
    ///     .build();
    /// // The compression threshold value is stored in csize.
    /// assert_eq!(header.csize, 0);
    /// ```
    pub fn threshold(&mut self, threshold: u32) -> &mut Self {
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
    /// * `chksum`: the new [Checksum](Checksum) algorithm to use for data verification.
    ///
    /// returns: SectionOptions
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::{Checksum, SectionOptions};
    /// use bpx::core::header::FLAG_CHECK_CRC32;
    ///
    /// let header = SectionOptions::new()
    ///     .checksum(Checksum::Crc32)
    ///     .build();
    /// assert_ne!(header.flags & FLAG_CHECK_CRC32, 0);
    /// ```
    pub fn checksum(&mut self, chksum: Checksum) -> &mut Self {
        match chksum {
            Checksum::Crc32 => self.header.flags |= FLAG_CHECK_CRC32,
            Checksum::Weak => self.header.flags |= FLAG_CHECK_WEAK,
        }
        self
    }

    /// Returns the generated [SectionHeader](SectionHeader).
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::{Checksum, CompressionMethod, SectionOptions};
    /// use bpx::core::header::{FLAG_CHECK_CRC32, FLAG_COMPRESS_ZLIB};
    ///
    /// let header = SectionOptions::new()
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
    pub fn build(&self) -> SectionHeader {
        self.header
    }
}

/// Utility to open an existing BPX [Container](crate::core::Container).
pub struct OpenOptions<T> {
    pub(crate) backend: T,
    pub(crate) skip_signature_check: bool,
    pub(crate) skip_checksum: bool,
    pub(crate) skip_version_check: bool,
    pub(crate) memory_threshold: u32,
    pub(crate) revert_on_save_fail: bool
}

impl<T> OpenOptions<T> {
     /// Creates a new set of options for a BPX container.
     ///
     /// # Arguments
     ///
     /// * `backend`: the IO backend to be associated with the container.
     ///
     /// returns: OpenOptions<T>
     pub fn new(backend: T) -> OpenOptions<T> {
        OpenOptions {
            backend,
            skip_checksum: false,
            skip_signature_check: false,
            skip_version_check: false,
            memory_threshold: DEFAULT_MEMORY_THRESHOLD,
            revert_on_save_fail: false
        }
    }

    /// Disable signature checks when loading the container.
    ///
    /// The default is set to false.
    ///
    /// # Arguments
    ///
    /// * `flag`: true to skip signature checks, false otherwise.
    ///
    /// returns: OpenOptions<T>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::OpenOptions;
    ///
    /// let header = OpenOptions::new(())
    ///     .skip_signature(true);
    /// ```
    pub fn skip_signature(mut self, flag: bool) -> Self {
        self.skip_signature_check = flag;
        self
    }

    /// Skip BPX version checks.
    ///
    /// The default is set to false.
    ///
    /// # Arguments
    ///
    /// * `flag`: true to skip version checks, false otherwise.
    ///
    /// returns: OpenOptions<T>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::OpenOptions;
    ///
    /// let header = OpenOptions::new(())
    ///     .skip_versions(true);
    /// ```
    pub fn skip_versions(mut self, flag: bool) -> Self {
        self.skip_version_check = flag;
        self
    }

    /// Disable checksum checks when loading the section header/table or a section.
    ///
    /// The default is set to false.
    ///
    /// # Arguments
    ///
    /// * `flag`: true to skip checksum checks on load, false otherwise.
    ///
    /// returns: OpenOptions<T>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::OpenOptions;
    ///
    /// let header = OpenOptions::new(())
    ///     .skip_checksum(true);
    /// ```
    pub fn skip_checksum(mut self, flag: bool) -> Self {
        self.skip_checksum = flag;
        self
    }

    /// Sets the maximum size of a section allowed to fit in RAM in bytes.
    ///
    /// The default is set to [DEFAULT_MEMORY_THRESHOLD](DEFAULT_MEMORY_THRESHOLD) bytes.
    ///
    /// # Arguments
    ///
    /// * `size`: the maximum size of a section (in bytes) in RAM.
    ///
    /// returns: CreateOptions<T>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::OpenOptions;
    ///
    /// let header = OpenOptions::new(())
    ///     .memory_threshold(4096); //Set memory threshold to 4Kb.
    /// ```
    pub fn memory_threshold(mut self, size: u32) -> Self {
        self.memory_threshold = size;
        self
    }

    /// Reverts the file when a save operation fails to keep the BPX unchanged/unmodified after
    /// a save failure.
    ///
    /// This works by saving the container to a temporary storage before overwriting the original
    /// IO backend. The default for this option is set to false.
    ///
    /// # Arguments
    ///
    /// * `flag`: true to revert the file on save failure, false otherwise.
    ///
    /// returns: OpenOptions<T>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::OpenOptions;
    ///
    /// let header = OpenOptions::new(())
    ///     .revert_on_save_failure(true);
    /// ```
    pub fn revert_on_save_failure(mut self, flag: bool) -> Self {
        self.revert_on_save_fail = flag;
        self
    }
}

impl<T: std::io::Seek> From<T> for OpenOptions<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// Utility to create a new BPX [Container](crate::core::Container) with a [MainHeader](MainHeader).
pub struct CreateOptions<T> {
    pub(crate) header: MainHeader,
    pub(crate) backend: T,
    pub(crate) memory_threshold: u32,
    pub(crate) revert_on_save_fail: bool
}

impl<T> CreateOptions<T> {
    /// Creates a new set of options for a BPX container.
    ///
    /// # Arguments
    ///
    /// * `backend`: the IO backend to be associated with the container.
    ///
    /// returns: OpenOptions<T>
    pub fn new(backend: T) -> CreateOptions<T> {
        CreateOptions {
            header: MainHeader::new(),
            backend,
            memory_threshold: DEFAULT_MEMORY_THRESHOLD,
            revert_on_save_fail: false
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
    /// returns: CreateOptions
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::CreateOptions;
    ///
    /// let header = CreateOptions::new(())
    ///     .ty('M' as u8)
    ///     .main_header();
    /// assert_eq!(header.ty, 'M' as u8);
    /// ```
    pub fn ty(mut self, ty: u8) -> Self {
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
    /// returns: CreateOptions
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::CreateOptions;
    ///
    /// let header = CreateOptions::new(())
    ///     .type_ext([1; 16])
    ///     .main_header();
    /// assert_eq!(header.type_ext.into_inner(), [1; 16]);
    /// ```
    pub fn type_ext(mut self, type_ext: impl Into<StaticByteBuf<16>>) -> Self {
        self.header.type_ext = type_ext.into();
        self
    }

    /// Defines the version of the BPX.
    ///
    /// *The default value of the version int is given by
    /// [BPX_CURRENT_VERSION](crate::core::header::BPX_CURRENT_VERSION).*
    ///
    /// **Note: A version which is not specified in [KNOWN_VERSIONS](crate::core::header::KNOWN_VERSIONS)
    /// will cause the decoder to fail loading the file, complaining that
    /// the file is corrupted.**
    ///
    /// # Arguments
    ///
    /// * `version`: the new version of the BPX.
    ///
    /// returns: CreateOptions
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::CreateOptions;
    ///
    /// let header = CreateOptions::new(())
    ///     .version(1)
    ///     .main_header();
    /// assert_eq!(header.version, 1);
    /// ```
    pub fn version(mut self, version: u32) -> Self {
        self.header.version = version;
        self
    }

    /// Sets the maximum size of a section allowed to fit in RAM in bytes.
    ///
    /// The default is set to [DEFAULT_MEMORY_THRESHOLD](DEFAULT_MEMORY_THRESHOLD) bytes.
    ///
    /// # Arguments
    ///
    /// * `size`: the maximum size of a section in RAM.
    ///
    /// returns: CreateOptions<T>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::CreateOptions;
    ///
    /// let header = CreateOptions::new(())
    ///     .memory_threshold(4096) //Set memory threshold to 4Kb.
    ///     .main_header();
    /// assert_eq!(header.version, 2);
    /// ```
    pub fn memory_threshold(mut self, size: u32) -> Self {
        self.memory_threshold = size;
        self
    }

    /// Reverts the file when a save operation fails to keep the BPX unchanged/unmodified after
    /// a save failure.
    ///
    /// This works by saving the container to a temporary storage before overwriting the original
    /// IO backend. The default for this option is set to false.
    ///
    /// # Arguments
    ///
    /// * `flag`: true to revert the file on save failure, false otherwise.
    ///
    /// returns: CreateOptions<T>
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::CreateOptions;
    ///
    /// let header = CreateOptions::new(())
    ///     .revert_on_save_failure(true);
    /// ```
    pub fn revert_on_save_failure(mut self, flag: bool) -> Self {
        self.revert_on_save_fail = flag;
        self
    }

    /// Returns the generated [MainHeader](MainHeader).
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::core::options::CreateOptions;
    /// use bytesutil::ByteBuf;
    ///
    /// let header = CreateOptions::new(())
    ///     .ty('M' as u8)
    ///     .type_ext([1; 16])
    ///     .version(1)
    ///     .main_header();
    /// assert_eq!(header.ty, 'M' as u8);
    /// assert_eq!(header.type_ext.into_inner(), [1; 16]);
    /// assert_eq!(header.version, 1);
    /// ```
    pub fn main_header(&self) -> MainHeader {
        self.header
    }
}

impl<T: std::io::Seek> From<T> for CreateOptions<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: std::io::Seek> From<(T, MainHeader)> for CreateOptions<T> {
    fn from((backend, header): (T, MainHeader)) -> Self {
        Self { header, backend, memory_threshold: DEFAULT_MEMORY_THRESHOLD, revert_on_save_fail: false }
    }
}

impl From<&mut SectionOptions> for SectionHeader {
    fn from(options: &mut SectionOptions) -> Self {
        options.build()
    }
}

impl From<SectionOptions> for SectionHeader {
    fn from(options: SectionOptions) -> Self {
        options.build()
    }
}
