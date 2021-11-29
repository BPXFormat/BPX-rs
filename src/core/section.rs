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

use std::io::{Read, Seek};
use std::ops::Deref;
use crate::core::data::AutoSectionData;
use crate::core::decoder::load_section1;
use crate::core::error::ReadError;
use crate::core::header::{FLAG_CHECK_CRC32, FLAG_CHECK_WEAK, FLAG_COMPRESS_XZ, FLAG_COMPRESS_ZLIB, SectionHeader};
use crate::Handle;
use crate::utils::OptionExtension;

pub struct SectionEntry1
{
    pub threshold: u32,
    pub flags: u8,
}

impl SectionEntry1
{
    pub fn get_flags(&self, size: u32) -> u8
    {
        let mut flags = 0;
        if self.flags & FLAG_CHECK_WEAK != 0 {
            flags |= FLAG_CHECK_WEAK;
        } else if self.flags & FLAG_CHECK_CRC32 != 0 {
            flags |= FLAG_CHECK_CRC32;
        }
        if self.flags & FLAG_COMPRESS_XZ != 0 && size > self.threshold {
            flags |= FLAG_COMPRESS_XZ;
        } else if self.flags & FLAG_COMPRESS_ZLIB != 0 && size > self.threshold {
            flags |= FLAG_COMPRESS_ZLIB;
        }
        flags
    }
}

pub struct SectionEntry
{
    pub entry1: SectionEntry1,
    pub header: SectionHeader,
    pub data: Option<AutoSectionData>,
    pub index: u32,
    pub modified: bool
}

pub struct SectionMut<'a, T>
{
    backend: &'a mut T,
    entry: &'a mut SectionEntry,
    handle: Handle
}

impl<'a, T: Read + Seek> SectionMut<'a, T>
{
    pub fn load(&mut self) -> Result<&mut AutoSectionData, ReadError>
    {
        let data = self.entry.data.get_or_insert_with_err(|| load_section1(self.backend, &self.entry.header))?;
        self.entry.modified = true;
        Ok(data)
    }
}

impl<'a, T> SectionMut<'a, T>
{
    pub fn open(&mut self) -> Option<&mut AutoSectionData>
    {
        self.entry.modified = true;
        self.entry.data.as_mut()
    }

    pub fn handle(&self) -> Handle
    {
        self.handle
    }

    pub fn index(&self) -> u32
    {
        self.entry.index
    }
}

impl<'a, T> Deref for SectionMut<'a, T>
{
    type Target = SectionHeader;

    fn deref(&self) -> &Self::Target
    {
        &self.entry.header
    }
}

pub fn new_section_mut<'a, T>(backend: &'a mut T, entry: &'a mut SectionEntry, handle: Handle) -> SectionMut<'a, T>
{
    SectionMut {
        backend,
        entry,
        handle
    }
}

pub struct Section<'a>
{
    entry: &'a SectionEntry,
    handle: Handle
}

impl<'a> Section<'a>
{
    pub fn open(&self) -> Option<&AutoSectionData>
    {
        self.entry.data.as_ref()
    }

    pub fn handle(&self) -> Handle
    {
        self.handle
    }

    pub fn index(&self) -> u32
    {
        self.entry.index
    }
}

impl<'a> Deref for Section<'a>
{
    type Target = SectionHeader;

    fn deref(&self) -> &Self::Target
    {
        &self.entry.header
    }
}

pub fn new_section(entry: &SectionEntry, handle: Handle) -> Section
{
    Section {
        entry,
        handle
    }
}
