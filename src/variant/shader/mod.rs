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

//! An implementation of the BPX type S (Shader) specification.

pub mod symbol;
mod decoder;
mod encoder;

pub use decoder::ShaderPackDecoder;
pub use encoder::ShaderPackBuilder;
pub use encoder::ShaderPackEncoder;

pub const SUPPORTED_VERSION: u32 = 0x2;

pub const SECTION_TYPE_SHADER: u8 = 0x1;
pub const SECTION_TYPE_SYMBOL_TABLE: u8 = 0x2;
pub const SECTION_TYPE_EXTENDED_DATA: u8 = 0x3;

#[derive(Clone)]
pub struct Shader
{
    stage: Stage,
    data: Vec<u8>
}

#[derive(Copy, Clone)]
pub enum Target
{
    DX11,
    DX12,
    GL33,
    GL40,
    VK10,
    MT,
    Any
}

#[derive(Copy, Clone)]
pub enum Type
{
    Assembly,
    Pipeline
}

#[derive(Copy, Clone)]
pub enum Stage
{
    Vertex,
    Hull,
    Domain,
    Geometry,
    Pixel
}
