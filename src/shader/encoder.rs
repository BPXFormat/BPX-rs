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

use byteorder::{ByteOrder, LittleEndian};

use crate::shader::{Settings, Target, Type};

pub fn get_type_ext(settings: &Settings) -> [u8; 16] {
    let mut type_ext: [u8; 16] = [0; 16];
    match settings.target {
        Target::DX11 => type_ext[10] = 0x1,
        Target::DX12 => type_ext[10] = 0x2,
        Target::GL33 => type_ext[10] = 0x3,
        Target::GL40 => type_ext[10] = 0x4,
        Target::GL41 => type_ext[10] = 0x5,
        Target::GL42 => type_ext[10] = 0x6,
        Target::GL43 => type_ext[10] = 0x7,
        Target::GL44 => type_ext[10] = 0x8,
        Target::GL45 => type_ext[10] = 0x9,
        Target::GL46 => type_ext[10] = 0xA,
        Target::ES30 => type_ext[10] = 0xB,
        Target::ES31 => type_ext[10] = 0xC,
        Target::ES32 => type_ext[10] = 0xD,
        Target::VK10 => type_ext[10] = 0xE,
        Target::VK11 => type_ext[10] = 0xF,
        Target::VK12 => type_ext[10] = 0x10,
        Target::MT => type_ext[10] = 0x11,
        Target::Any => type_ext[10] = 0xFF,
    };
    match settings.ty {
        Type::Assembly => type_ext[11] = b'A',
        Type::Pipeline => type_ext[11] = b'P',
    };
    LittleEndian::write_u64(&mut type_ext[0..8], settings.assembly_hash);
    type_ext
}
