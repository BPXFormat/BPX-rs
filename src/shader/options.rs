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

use crate::{
    macros::{create_options, open_options},
    shader::{Target, Type},
};

/// The required settings to create a new BPXS.
///
/// *This is intended to be generated with help of [CreateOptions](CreateOptions).*
#[derive(Clone)]
pub struct Settings {
    /// The assembly hash of the shader package.
    pub assembly_hash: u64,

    /// The target rendering API of the shader package.
    pub target: Target,

    /// The type of the shader package (Assembly or Pipeline).
    pub ty: Type,
}

create_options! {
    /// Utility to simplify generation of [Settings](Settings) required when creating a new BPXS.
    CreateOptions {
        settings: Settings = Settings {
            assembly_hash: 0,
            target: Target::Any,
            ty: Type::Pipeline,
        }
    }
}

impl<T> CreateOptions<T> {
    /// Defines the shader assembly this package is linked against.
    ///
    /// *By default, no shader assembly is linked and the hash is 0.*
    ///
    /// # Arguments
    ///
    /// * `hash`: the shader assembly hash.
    pub fn assembly(mut self, hash: u64) -> Self {
        self.settings.assembly_hash = hash;
        self
    }

    /// Defines the target of this shader package.
    ///
    /// *By default, the target is Any.*
    ///
    /// # Arguments
    ///
    /// * `target`: the shader target.
    pub fn target(mut self, target: Target) -> Self {
        self.settings.target = target;
        self
    }

    /// Defines the shader package type.
    ///
    /// *By default, the type is Pipeline.*
    ///
    /// # Arguments
    ///
    /// * `ty`: the shader package type (pipeline/program or assembly).
    pub fn ty(mut self, ty: Type) -> Self {
        self.settings.ty = ty;
        self
    }
}

impl<T: std::io::Seek> From<(T, Settings)> for CreateOptions<T> {
    fn from((backend, settings): (T, Settings)) -> Self {
        Self {
            options: crate::core::options::CreateOptions::new(backend),
            settings,
        }
    }
}

open_options! {
    /// Utility to allow configuring the underlying BPX container when opening a BPXS.
    OpenOptions {}
}
