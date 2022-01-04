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

use crate::{
    package::{Architecture, Platform},
    sd::Object
};

/// The required settings to create a new BPXP.
///
/// *This is intended to be generated with help of [Builder](crate::package::Builder).*
#[derive(Clone)]
pub struct Settings
{
    /// The package target architecture.
    pub architecture: Architecture,

    /// The package target platform/OS.
    pub platform: Platform,

    /// The package metadata (stored as a BPXSD [Object](crate::sd::Object)).
    pub metadata: Option<Object>,

    /// The package type code.
    pub type_code: [u8; 2]
}

/// Utility to simplify generation of [Settings](crate::package::Settings) required when creating a new BPXP.
pub struct Builder
{
    settings: Settings
}

impl Default for Builder
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl Builder
{
    /// Creates a new BPX Package builder.
    pub fn new() -> Builder
    {
        Builder {
            settings: Settings {
                architecture: Architecture::Any,
                platform: Platform::Any,
                metadata: None,
                type_code: [0x50, 0x48]
            }
        }
    }

    /// Defines the CPU architecture that the package is targeting.
    ///
    /// *By default, no CPU architecture is targeted.*
    ///
    /// # Arguments
    ///
    /// * `arch`: the CPU architecture this package is designed to work on.
    ///
    /// returns: PackageBuilder
    pub fn architecture(&mut self, arch: Architecture) -> &mut Self
    {
        self.settings.architecture = arch;
        self
    }

    /// Defines the platform that the package is targeting.
    ///
    /// *By default, no platform is targeted.*
    ///
    /// # Arguments
    ///
    /// * `platform`: the platform this package is designed to work on.
    ///
    /// returns: PackageBuilder
    pub fn platform(&mut self, platform: Platform) -> &mut Self
    {
        self.settings.platform = platform;
        self
    }

    /// Defines the metadata for the package.
    ///
    /// *By default, no metadata object is set.*
    ///
    /// # Arguments
    ///
    /// * `obj`: the BPXSD metadata object.
    ///
    /// returns: PackageBuilder
    pub fn metadata(&mut self, obj: Object) -> &mut Self
    {
        self.settings.metadata = Some(obj);
        self
    }

    /// Defines the type of the package.
    ///
    /// *By default, the package variant is 'PK' to identify
    /// a package designed for FPKG.*
    ///
    /// # Arguments
    ///
    /// * `type_code`: the type code of this package.
    ///
    /// returns: PackageBuilder
    pub fn type_code(&mut self, type_code: [u8; 2]) -> &mut Self
    {
        self.settings.type_code = type_code;
        self
    }

    /// Returns the built settings.
    pub fn build(&self) -> Settings
    {
        self.settings.clone()
    }
}

impl From<&mut Builder> for Settings
{
    fn from(builder: &mut Builder) -> Self
    {
        builder.build()
    }
}

impl From<Builder> for Settings
{
    fn from(builder: Builder) -> Self
    {
        builder.build()
    }
}
