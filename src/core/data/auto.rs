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

use std::io::{Read, Seek, SeekFrom, Write};

use tempfile::tempfile;

use crate::core::{
    data::{file::FileBasedSection, memory::InMemorySection},
    SectionData
};

const MEMORY_THRESHOLD: u32 = 100000000;
const INIT_BUF_SIZE: usize = 512;

enum DynSectionData
{
    File(FileBasedSection),
    Memory(InMemorySection)
}

pub struct AutoSectionData
{
    inner: DynSectionData
}

impl Default for AutoSectionData
{
    fn default() -> Self
    {
        Self::new()
    }
}

impl AutoSectionData
{
    pub fn new() -> AutoSectionData
    {
        AutoSectionData {
            inner: DynSectionData::Memory(InMemorySection::new(512))
        }
    }

    pub fn new_with_size(size: u32) -> std::io::Result<AutoSectionData>
    {
        if size >= MEMORY_THRESHOLD {
            let file = FileBasedSection::new(tempfile()?);
            Ok(AutoSectionData {
                inner: DynSectionData::File(file)
            })
        } else {
            Ok(AutoSectionData {
                inner: DynSectionData::Memory(InMemorySection::new(size as usize))
            })
        }
    }

    unsafe fn move_to_file(&mut self) -> std::io::Result<()>
    {
        let mut file = FileBasedSection::new(tempfile()?);
        match &mut self.inner {
            DynSectionData::Memory(m) => std::io::copy(m, &mut file),
            //SAFETY: If the section is not an InMemorySection then move_to_file is not supposed to have been called,
            // and that is an unrecoverable internal BPX error
            DynSectionData::File(_) => std::hint::unreachable_unchecked()
        }?;
        self.inner = DynSectionData::File(file);
        Ok(())
    }

    pub fn clear(&mut self)
    {
        self.inner = DynSectionData::Memory(InMemorySection::new(INIT_BUF_SIZE))
    }
}

impl Read for AutoSectionData
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>
    {
        match &mut self.inner {
            DynSectionData::File(f) => f.read(buf),
            DynSectionData::Memory(m) => m.read(buf)
        }
    }
}

impl Write for AutoSectionData
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize>
    {
        match &mut self.inner {
            DynSectionData::File(f) => f.write(buf),
            DynSectionData::Memory(m) => {
                let size = m.write(buf)?;
                if m.size() >= MEMORY_THRESHOLD as usize {
                    unsafe {
                        self.move_to_file()?;
                    }
                }
                Ok(size)
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()>
    {
        match &mut self.inner {
            DynSectionData::File(f) => f.flush(),
            DynSectionData::Memory(m) => m.flush()
        }
    }
}

impl Seek for AutoSectionData
{
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64>
    {
        match &mut self.inner {
            DynSectionData::File(f) => f.seek(pos),
            DynSectionData::Memory(m) => m.seek(pos)
        }
    }
}

impl SectionData for AutoSectionData
{
    fn load_in_memory(&mut self) -> std::io::Result<Vec<u8>>
    {
        match &mut self.inner {
            DynSectionData::File(f) => f.load_in_memory(),
            DynSectionData::Memory(m) => m.load_in_memory()
        }
    }

    fn size(&self) -> usize
    {
        match &self.inner {
            DynSectionData::File(f) => f.size(),
            DynSectionData::Memory(m) => m.size()
        }
    }
}
