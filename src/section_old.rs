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

use byteorder::LittleEndian;
use byteorder::ByteOrder;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::fs::File;
use std::boxed::Box;
use std::num::Wrapping;

use crate::compression::EasyChecksum;
use crate::compression::Checksum;
use crate::compression::XzCompressionMethod;
use crate::compression::Inflater;
use crate::compression::Deflater;
use crate::header::SectionHeader;

pub const SIZE_SECTION_HEADER: usize = 24;

pub trait SectionData : io::Read + io::Write + io::Seek
{
    fn load_in_memory(&mut self) -> io::Result<Vec<u8>>;
    fn size(&self) -> usize; //The computed size of the section
}



fn read_chksum(data: &[u8]) -> Wrapping<u32>
{
    let mut chk: Wrapping<u32> = Wrapping(0);

    for i in 0..data.len()
    {
        chk += Wrapping(data[i] as u32);
    }
    return chk;
}

const FLAG_COMPRESS_XZ: u8 = 0x2;
const FLAG_CHECK_WEAK: u8 = 0x8;
const READ_BLOCK_SIZE: usize = 8192;

fn block_based_deflate(input: &mut dyn Read, output: &mut dyn Write, inflated_size: usize) -> io::Result<(usize, u32)>
{
    let mut chksum = EasyChecksum::new();
    let size = XzCompressionMethod::deflate(input, output, inflated_size, &mut chksum)?;
    return Ok((size, chksum.finish()));
}

fn block_based_inflate(input: &mut dyn Read, output: &mut dyn Write, deflated_size: usize) -> io::Result<u32>
{
    let mut chksum = EasyChecksum::new();
    XzCompressionMethod::inflate(input, output, deflated_size, &mut chksum)?;
    return Ok(chksum.finish());    
}

fn load_section_in_memory(bpx: &mut File, header: &SectionHeader) -> io::Result<InMemorySection>
{
    bpx.seek(io::SeekFrom::Start(header.pointer))?;
    if header.flags & FLAG_COMPRESS_XZ == FLAG_COMPRESS_XZ
    {
        let mut section = InMemorySection::new(vec![0; header.size as usize]);
        section.seek(io::SeekFrom::Start(0))?;
        let chksum = block_based_inflate(bpx, &mut section, header.csize as usize)?;
        println!("Unpacked section size: {}", section.size());
        if header.flags & FLAG_CHECK_WEAK == FLAG_CHECK_WEAK && chksum != header.chksum
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("[BPX] checksum validation failed {} != {}", chksum, header.chksum)));
        }
        section.seek(io::SeekFrom::Start(0))?;
        return Ok(section);
    }
    else
    {
        let mut data = vec![0; header.size as usize];
        bpx.read(&mut data)?;
        let chksum = read_chksum(&data);
        if header.flags & FLAG_CHECK_WEAK == FLAG_CHECK_WEAK && chksum.0 != header.chksum
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("[BPX] checksum validation failed {} != {}", chksum, header.chksum)));
        }
        let mut section = InMemorySection::new(data);
        section.seek(io::SeekFrom::Start(0))?;
        return Ok(section);
    }
}

fn load_section_as_file(bpx: &mut File, header: &SectionHeader) -> io::Result<FileBasedSection>
{
    let mut section = FileBasedSection::new(tempfile::tempfile()?);

    bpx.seek(io::SeekFrom::Start(header.pointer))?;
    if header.flags & FLAG_COMPRESS_XZ == FLAG_COMPRESS_XZ
    {
        let chksum = block_based_inflate(bpx, &mut section, header.csize as usize)?;
        println!("Unpacked section size: {}", section.size());
        if header.flags & FLAG_CHECK_WEAK == FLAG_CHECK_WEAK && chksum != header.chksum
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("[BPX] checksum validation failed {} != {}", chksum, header.chksum)));
        }
    }
    else
    {
        let mut idata: [u8; READ_BLOCK_SIZE] = [0; READ_BLOCK_SIZE];
        let mut count: usize = 0;
        let mut chksum: Wrapping<u32> = Wrapping(0);
        let mut remaining: usize = header.size as usize;
        while count < header.size as usize
        {
            let res = bpx.read(&mut idata[0..std::cmp::min(READ_BLOCK_SIZE, remaining)])?;
            section.write(&idata[0..res])?;
            chksum += read_chksum(&idata[0..res]);
            count += res;
            remaining -= res;
        }
        if header.flags & FLAG_CHECK_WEAK == FLAG_CHECK_WEAK && chksum.0 != header.chksum
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("[BPX] checksum validation failed {} != {}", chksum, header.chksum)));
        }
        section.flush()?;
    }
    section.seek(io::SeekFrom::Start(0))?;
    return Ok(section);
}

pub fn open_section(bpx: &mut File, header: &SectionHeader) -> io::Result<Box<dyn SectionData>>
{
    if header.is_huge_section()
    {
        let data = load_section_as_file(bpx, &header)?;
        return Ok(Box::from(data));
    }
    else
    {
        let data = load_section_in_memory(bpx, &header)?;
        return Ok(Box::from(data));
    }
}

