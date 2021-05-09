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

use std::fs::File;
use std::path::Path;
use std::io;
use std::io::Write;

use crate::header::MainHeader;
use crate::header::SectionHeader;
use crate::header::FLAG_COMPRESS_ZLIB;
use crate::header::FLAG_COMPRESS_XZ;
use crate::header::FLAG_CHECK_CRC32;
use crate::header::FLAG_CHECK_WEAK;
use crate::section::SectionData;
use crate::section::new_section_data;
use crate::compression::XzCompressionMethod;
use crate::compression::Checksum;
use crate::compression::EasyChecksum;
use crate::compression::Inflater;
use crate::BPX;
use crate::SectionHandle;
use crate::OptionExtension;

const READ_BLOCK_SIZE: usize = 8192;

pub struct Decoder
{
    pub main_header: MainHeader,
    sections: Vec<SectionHeader>,
    sections_data: Vec<Option<Box<dyn SectionData>>>,
    file: File
}

impl Decoder
{
    fn read_section_header_table(&mut self, checksum: u32) -> io::Result<()>
    {
        let mut final_checksum = checksum;

        for _ in 0..self.main_header.section_num
        {
            let (checksum, header) = SectionHeader::read(&mut self.file)?;
            if header.flags & FLAG_COMPRESS_ZLIB == FLAG_COMPRESS_ZLIB
            {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "[BPX] zlib compression is not supported by FPKG"));
            }
            if header.flags & FLAG_CHECK_CRC32 == FLAG_CHECK_CRC32
            {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "[BPX] crc32 checksum is not supported by FPKG"));
            }
            final_checksum += checksum;
            self.sections.push(header);
        }
        if final_checksum != self.main_header.chksum
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "[BPX] checksum validation failed"));
        }
        return Ok(());
    }

    /*pub fn load_string_section(&mut self) -> io::Result<Box<dyn SectionData>>
    {
        if let Some(section) = self.find_section_by_type(SECTION_TYPE_STRING)
        {
            return self.open_section(&section);
        }
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "[BPX] could not locate string section"));
    }*/

    pub fn new(file: &Path) -> io::Result<Decoder>
    {
        let mut fle = File::open(file)?;
        let (checksum, header) = MainHeader::read(&mut fle)?;
        let num = header.section_num;
        let mut decoder = Decoder
        {
            file: fle,
            main_header: header,
            sections: Vec::with_capacity(num as usize),
            sections_data: std::iter::repeat_with(|| None).take(num as usize).collect()
        };
        decoder.read_section_header_table(checksum)?;
        return Ok(decoder);
    }
}

impl BPX for Decoder
{
    fn find_section_by_type(&self, btype: u8) -> Option<SectionHandle>
    {
        for i in 0..self.sections.len()
        {
            if self.sections[i].btype == btype
            {
                return Some(i);
            }
        }
        return None;
    }

    fn find_all_sections_of_type(&self, btype: u8) -> Vec<SectionHandle>
    {
        let mut v = Vec::new();

        for i in 0..self.sections.len()
        {
            if self.sections[i].btype == btype
            {
                v.push(i);
            }
        }
        return v;
    }

    fn find_section_by_index(&self, index: u32) -> Option<SectionHandle>
    {
        if let Some(_) = self.sections.get(index as usize)
        {
            return Some(index as SectionHandle);
        }
        return None;
    }

    fn get_section_header(&self, handle: SectionHandle) -> &SectionHeader
    {
        return &self.sections[handle];
    }

    fn open_section(&mut self, handle: SectionHandle) -> io::Result<&mut dyn SectionData>
    {
        let header = &self.sections[handle];
        let file = &mut self.file;
        let object = self.sections_data[handle].get_or_insert_with_err(|| load_section(file, header))?;
        return Ok(object.as_mut());
    }
}

fn load_section<TBpx: io::Seek + io::Read>(file: &mut TBpx, section: &SectionHeader) -> io::Result<Box<dyn SectionData>>
{
    let mut chksum = EasyChecksum::new();
    if section.flags & FLAG_CHECK_CRC32 != 0
    {
        //TODO: Implement CRC checksum
        panic!("[BPX] CRC32 check not yet supported!");
    }
    let mut data = new_section_data(Some(section.size))?;
    data.seek(io::SeekFrom::Start(0))?;
    if section.flags & FLAG_COMPRESS_XZ != 0
    {
        load_section_compressed::<XzCompressionMethod, _>(file, &section, &mut data, &mut chksum)?;
    }
    else if section.flags & FLAG_COMPRESS_ZLIB != 0
    {
        //TODO: Implement Zlib compression
        panic!("[BPX] ZLib compression not yet supported!");
    }
    else
    {
        load_section_uncompressed(file, &section, &mut data, &mut chksum)?;
    }
    let v = chksum.finish();
    if (section.flags & FLAG_CHECK_CRC32 != 0 || section.flags & FLAG_CHECK_WEAK != 0) && v != section.chksum
    {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("[BPX] checksum validation failed {} != {}", v, section.chksum)));
    }
    return Ok(data);
}

fn load_section_uncompressed<TBpx: io::Read + io::Seek>(bpx: &mut TBpx, header: &SectionHeader, output: &mut dyn Write, chksum: &mut dyn Checksum) -> io::Result<()>
{
    let mut idata: [u8; READ_BLOCK_SIZE] = [0; READ_BLOCK_SIZE];
    let mut count: usize = 0;
    let mut remaining: usize = header.size as usize;

    bpx.seek(io::SeekFrom::Start(header.pointer))?;
    while count < header.size as usize
    {
        let res = bpx.read(&mut idata[0..std::cmp::min(READ_BLOCK_SIZE, remaining)])?;
        output.write(&idata[0..res])?;
        chksum.push(&idata[0..res]);
        count += res;
        remaining -= res;
    }
    return Ok(());
}

fn load_section_compressed<TMethod: Inflater, TBpx: io::Read + io::Seek>(bpx: &mut TBpx, header: &SectionHeader, output: &mut dyn Write, chksum: &mut dyn Checksum) -> io::Result<()>
{
    bpx.seek(io::SeekFrom::Start(header.pointer))?;
    XzCompressionMethod::inflate(bpx, output, header.size as usize, chksum)?;
    return Ok(());
}
