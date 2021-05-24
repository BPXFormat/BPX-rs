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

//! An implementation of the BPX type P (Package) specification

pub mod encoder;
pub mod decoder;

/// The standard type for a data section in a BPX Package (type P)
pub const DATA_SECTION_TYPE: u8 = 0x1;

/// Enum of all supported processor architectures by BPXP
#[derive(Clone, Copy)]
pub enum Architecture
{
    /// x86_64
    ///
    /// *x86 extension for 64 bits originally made by AMD*
    ///
    /// *This architecture is now the standard for all new desktops and laptops*
    X86_64,

    /// aarch64
    ///
    /// *commonly known as ARM64 (64 bits)*
    ///
    /// *This architecture is usually found in newer smartphones and some embedded devices*
    Aarch64,

    /// x86
    ///
    /// *original Intel architecture*
    ///
    /// *The predecessor of x86_64*
    X86,

    /// armv7hl
    ///
    /// *commonly known as ARM (32 bits)*
    ///
    /// *This architecture is usually found in older smartphones and other embedded devices*
    Armv7hl,

    /// The package does not have a target architecture and by extension can be loaded on any CPU
    Any
}

/// Enum of all supported platforms by BPXP
#[derive(Clone, Copy)]
pub enum Platform
{
    /// GNU / Linux
    ///
    /// *matches any distribution with or without desktop environment*
    Linux,

    /// Mac OS
    ///
    /// *if this platform is set alongside x86_64 architecture, Mac OS X is assumed*
    ///
    /// *if this platform is set alongside aarch64 architecture, Apple Silicon with Mac OS 11 (Big Sur) is assumed*
    Mac,

    /// Windows
    ///
    /// *refers to Windows 7 or later, compatibility with Windows XP and older is not guaranteed*
    Windows,

    /// Android OS based on a Linux kernel
    ///
    /// *refers to Android API level 21+, compatibility with older versions is not guaranteed*
    Android,

    /// The package does not have a target platform and by extension can be loaded on any platform
    Any
}
