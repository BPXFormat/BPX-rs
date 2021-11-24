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

mod decoder;
mod encoder;
pub mod error;
pub mod symbol;

pub use decoder::ShaderPackDecoder;
pub use encoder::{ShaderPackBuilder, ShaderPackEncoder};

/// The supported BPX version for this shader variant decoder/encoder.
pub const SUPPORTED_VERSION: u32 = 0x2;

/// The standard type for a shader section in a BPX Shader Package (type S).
pub const SECTION_TYPE_SHADER: u8 = 0x1;

/// The standard type for a symbol table section in a BPX Shader Package (type S).
pub const SECTION_TYPE_SYMBOL_TABLE: u8 = 0x2;

/// The standard type for an extended data section in a BPX Shader Package (type S).
pub const SECTION_TYPE_EXTENDED_DATA: u8 = 0x3;

/// Represents a shader in a BPXS.
#[derive(Clone, Debug)]
pub struct Shader
{
    /// The shader stage.
    pub stage: Stage,

    /// The shader data.
    pub data: Vec<u8>
}

/// Enum of all supported shader targets by BPXS.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
pub enum Target
{
    /// DirectX 11
    DX11,

    /// DirectX 12
    DX12,

    /// OpenGL 3.3 (Core context)
    GL33,

    /// OpenGL 4.0 (Core context)
    GL40,

    /// OpenGL 4.1 (Core context)
    GL41,

    /// OpenGL 4.2 (Core context)
    GL42,

    /// OpenGL 4.3 (Core context)
    GL43,

    /// OpenGL 4.4 (Core context)
    GL44,

    /// OpenGL 4.5 (Core context)
    GL45,

    /// OpenGL 4.6 (Core context)
    GL46,

    /// OpenGL ES 3.0
    ES30,

    /// OpenGL ES 3.1
    ES31,

    /// OpenGL ES 3.2
    ES32,

    /// Vulkan 1.0
    VK10,

    /// Vulkan 1.1
    VK11,

    /// Vulkan 1.2
    VK12,

    /// Apple Metal
    MT,

    /// Any rendering API. Useful if this is a shader assembly.
    Any
}

/// Enum of all types of BPXS.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
pub enum Type
{
    /// A shader assembly.
    Assembly,

    /// A shader pipeline/program.
    Pipeline
}

/// Enum of all supported shader stages by BPXS.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
pub enum Stage
{
    /// Vertex shader stage.
    Vertex,

    /// Hull/Tessellation Control Shader (TCS) stage.
    Hull,

    /// Domain/Tessellation Evaluation Shader (TES) stage.
    Domain,

    /// Geometry shader stage.
    Geometry,

    /// Pixel/fragment shader stage.
    Pixel
}
