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

use crate::core::builder::{Checksum, CompressionMethod, SectionHeaderBuilder};
use crate::core::header::SectionHeader;
use crate::package::{Architecture, Platform, SECTION_TYPE_DATA, Settings};

pub fn create_data_section_header() -> SectionHeader
{
    SectionHeaderBuilder::new()
        .with_type(SECTION_TYPE_DATA)
        .with_compression(CompressionMethod::Xz)
        .with_checksum(Checksum::Crc32)
        .build()
}

pub fn get_type_ext(settings: &Settings) -> [u8; 16]
{
    let mut type_ext: [u8; 16] = [0; 16];
    match settings.architecture {
        Architecture::X86_64 => type_ext[0] = 0x0,
        Architecture::Aarch64 => type_ext[0] = 0x1,
        Architecture::X86 => type_ext[0] = 0x2,
        Architecture::Armv7hl => type_ext[0] = 0x3,
        Architecture::Any => type_ext[0] = 0x4
    }
    match settings.platform {
        Platform::Linux => type_ext[1] = 0x0,
        Platform::Mac => type_ext[1] = 0x1,
        Platform::Windows => type_ext[1] = 0x2,
        Platform::Android => type_ext[1] = 0x3,
        Platform::Any => type_ext[1] = 0x4
    }
    type_ext[2] = settings.type_code[0];
    type_ext[3] = settings.type_code[1];
    type_ext
}
