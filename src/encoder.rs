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
use std::io::Seek;
use std::io::Read;
use std::io::Write;

use crate::header::MainHeader;
use crate::header::SectionHeader;
use crate::section::SectionData;
use crate::header::SIZE_MAIN_HEADER;
use crate::header::SIZE_SECTION_HEADER;
use crate::header::FLAG_CHECK_WEAK;
use crate::header::FLAG_CHECK_CRC32;
use crate::header::FLAG_COMPRESS_XZ;
use crate::header::FLAG_COMPRESS_ZLIB;
use crate::section::new_section_data;
use crate::compression::Checksum;
use crate::compression::Deflater;
use crate::compression::EasyChecksum;
use crate::compression::XzCompressionMethod;
use crate::Interface;
use crate::SectionHandle;

const READ_BLOCK_SIZE: usize = 8192;

pub struct Encoder
{
    main_header: MainHeader,
    sections: Vec<SectionHeader>,
    sections_data: Vec<Box<dyn SectionData>>,
    file: File
}

impl Encoder
{
    pub fn new(file: &Path) -> io::Result<Encoder>
    {
        let fle = File::create(file)?;
        return Ok(Encoder
        {
            main_header: MainHeader::new(),
            sections: Vec::new(),
            sections_data: Vec::new(),
            file: fle
        });
    }

    pub fn set_main_header(&mut self, main_header: MainHeader)
    {
        self.main_header = main_header;
    }

    //Adds a new section; returns a reference to the new section for use in edit_section
    pub fn create_section(&mut self, header: SectionHeader) -> io::Result<SectionHandle>
    {
        self.main_header.section_num += 1;
        let section = create_section(&header)?;
        self.sections.push(header);
        let r = self.sections.len() - 1;
        self.sections_data.push(section);
        return Ok(r);
    }

    fn write_sections(&mut self) -> io::Result<(File, u32, usize)>
    {
        let mut all_sections_size: usize = 0;
        let mut chksum_sht: u32 = 0;
        let mut ptr: u64 = SIZE_MAIN_HEADER as u64 + (self.sections.len() as u64 * SIZE_SECTION_HEADER as u64);
        let mut f = tempfile::tempfile()?;

        for i in 0..self.sections.len()
        {
            if self.sections_data[i].size() > u32::MAX as usize
            {
                panic!("BPX cannot support individual sections with size exceeding 4Gb (2 pow 32)");
            }
            self.sections_data[i].seek(io::SeekFrom::Start(0))?;
            let mut chksum = EasyChecksum::new();
            let csize;
            let flags = get_flags(&self.sections[i], self.sections_data[i].size() as u32);
            if flags & FLAG_COMPRESS_XZ != 0
            {
                csize = write_section_compressed::<XzCompressionMethod>(self.sections_data[i].as_mut(), &mut f, &mut chksum)?;
            }
            else if flags & FLAG_COMPRESS_ZLIB != 0
            {
                //TODO: Implement Zlib compression
                panic!("[BPX] ZLib compression not yet supported!");
            }
            else
            {
                csize = write_section_uncompressed(self.sections_data[i].as_mut(), &mut f, &mut chksum)?;
            }
            self.sections[i].csize = csize as u32;
            self.sections[i].size = self.sections_data[i].size() as u32;
            self.sections[i].chksum = chksum.finish();
            self.sections[i].flags = flags;
            self.sections[i].pointer = ptr;
            println!("Writing section #{}: Size = {}, Size after compression = {}", i, self.sections[i].size, self.sections[i].csize);
            ptr += csize as u64;
            chksum_sht += self.sections[i].get_checksum();
            all_sections_size += csize;
        }
        return Ok((f, chksum_sht, all_sections_size));
    }

    fn write_data_file(&mut self, fle: &mut File, all_sections_size: usize) -> io::Result<()>
    {
        let mut idata: [u8; 8192] = [0; 8192];
        let mut count: usize = 0;

        fle.seek(io::SeekFrom::Start(0))?;
        while count < all_sections_size
        {
            let res = fle.read(&mut idata)?;
            self.file.write(&idata[0..res])?;
            count += res;
        }
        return Ok(());
    }

    pub fn save(&mut self) -> io::Result<()>
    {
        let (mut main_data, chksum_sht, all_sections_size) = self.write_sections()?;

        self.main_header.file_size = all_sections_size as u64 + (self.sections.len() * SIZE_SECTION_HEADER) as u64 + SIZE_MAIN_HEADER as u64;
        self.main_header.chksum = chksum_sht + self.main_header.get_checksum();
        self.main_header.write(&mut self.file)?;
        for v in &self.sections
        {
            v.write(&mut self.file)?;
        }
        self.write_data_file(&mut main_data, all_sections_size)?;
        return Ok(());
    }
}

impl Interface for Encoder
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
        return Ok(self.sections_data[handle].as_mut());
    }

    fn get_main_header(&self) -> &MainHeader
    {
        return &self.main_header;
    }
}

fn get_flags(header: &SectionHeader, size: u32) -> u8
{
    let mut flags = 0;
    if header.flags & FLAG_CHECK_WEAK != 0
    {
        flags |= FLAG_CHECK_WEAK;
    }
    else if header.flags & FLAG_CHECK_CRC32 != 0
    {
        flags |= FLAG_CHECK_CRC32;
    }
    if header.flags & FLAG_COMPRESS_XZ != 0 && size > header.csize
    {
        flags |= FLAG_COMPRESS_XZ;
    }
    else if header.flags & FLAG_COMPRESS_ZLIB != 0 && size > header.csize
    {
        flags |= FLAG_COMPRESS_ZLIB;
    }
    return flags;
}

fn create_section(header: &SectionHeader) -> io::Result<Box<dyn SectionData>>
{
    if header.size == 0
    {
        let mut section = new_section_data(None)?;
        section.seek(io::SeekFrom::Start(0))?;
        return Ok(section);
    }
    else
    {
        let mut section = new_section_data(Some(header.size))?;
        section.seek(io::SeekFrom::Start(0))?;
        return Ok(section);
    }
}

fn write_section_uncompressed(section: &mut dyn SectionData, out: &mut dyn Write, chksum: &mut dyn Checksum) -> io::Result<usize>
{
    let mut idata: [u8; READ_BLOCK_SIZE] = [0; READ_BLOCK_SIZE];
    let mut count: usize = 0;
    while count < section.size() as usize
    {
        let res = section.read(&mut idata)?;
        out.write(&idata[0..res])?;
        chksum.push(&idata[0..res]);
        count += res;
    }
    section.flush()?;
    return Ok(section.size());
}

fn write_section_compressed<TMethod: Deflater>(mut section: &mut dyn SectionData, out: &mut dyn Write, chksum: &mut dyn Checksum) -> io::Result<usize>
{
    let size = section.size();
    let csize = TMethod::deflate(&mut section, out, size, chksum)?;
    return Ok(csize);
}
