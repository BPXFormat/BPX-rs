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

use super::garraylen::*;

pub const SIZE_MAIN_HEADER: usize = 40;
pub const SIZE_SECTION_HEADER: usize = 24;

pub const FLAG_COMPRESS_XZ: u8 = 0x2;
pub const FLAG_CHECK_WEAK: u8 = 0x8;
pub const FLAG_COMPRESS_ZLIB: u8 = 0x1;
pub const FLAG_CHECK_CRC32: u8 = 0x4;

pub const SECTION_TYPE_STRING: u8 = 0xFF;
pub const SECTION_TYPE_SD: u8 = 0xFE;

#[derive(Copy, Clone)]
pub struct MainHeader
{
    pub signature: [u8; 3], //+0
    pub btype: u8, //+3
    pub chksum: u32, //+4
    pub file_size: u64, //+8
    pub section_num: u32, //+16
    pub version: u32, //+20
    pub type_ext: [u8; 16] //+24
}

impl MainHeader
{
    pub fn read<TReader: io::Read>(reader: &mut TReader) -> io::Result<(u32, MainHeader)>
    {
        let mut buf: [u8;SIZE_MAIN_HEADER] = [0;SIZE_MAIN_HEADER];
        let mut checksum: u32 = 0;

        reader.read(&mut buf)?;
        for i in 0..SIZE_MAIN_HEADER
        {
            if i < 4 || i > 7
            {
                checksum += buf[i] as u32;
            }
        }
        let head = MainHeader {
            signature: extract_slice::<T3>(&buf, 0),
            btype: buf[3],
            chksum: LittleEndian::read_u32(&buf[4..8]),
            file_size: LittleEndian::read_u64(&buf[8..16]),
            section_num: LittleEndian::read_u32(&buf[16..20]),
            version: LittleEndian::read_u32(&buf[20..24]),
            type_ext: extract_slice::<T16>(&buf, 24)
        };
        if head.signature[0] != 'B' as u8 || head.signature[1] != 'P' as u8 || head.signature[2] != 'X' as u8
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "[BPX] File is not a BPX: incorrect signature"));
        }
        if head.version != 0x1
        {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("[BPX] Unsupported version of BPX: {}", head.version)));
        }
        return Ok((checksum, head));
    }

    pub fn new() -> MainHeader
    {
        return MainHeader
        {
            signature: ['B' as u8, 'P' as u8, 'X' as u8], //+0
            btype: 'P' as u8, //+3
            chksum: 0, //+4
            file_size: SIZE_MAIN_HEADER as u64, //+8
            section_num: 0, //+16
            version: 0x1, //+20
            type_ext: [0; 16]
        }
    }

    fn to_bytes(&self) -> [u8; SIZE_MAIN_HEADER]
    {
        let mut block: [u8; SIZE_MAIN_HEADER] = [0; SIZE_MAIN_HEADER];
        block[0] = self.signature[0];
        block[1] = self.signature[1];
        block[2] = self.signature[2];
        block[3] = self.btype;
        LittleEndian::write_u32(&mut block[4..8], self.chksum);
        LittleEndian::write_u64(&mut block[8..16], self.file_size);
        LittleEndian::write_u32(&mut block[16..20], self.section_num);
        LittleEndian::write_u32(&mut block[20..24], self.version);
        for i in 24..40
        {
            block[i] = self.type_ext[i - 24];
        }
        return block;
    }

    pub fn get_checksum(&self) -> u32
    {
        let mut checksum: u32 = 0;
        let buf = self.to_bytes();
        for i in 0..SIZE_MAIN_HEADER
        {
            checksum += buf[i] as u32;
        }
        return checksum;
    }

    pub fn write<TWriter: io::Write>(&self, writer: &mut TWriter) -> io::Result<()>
    {
        let buf = self.to_bytes();
        writer.write(&buf)?;
        writer.flush()?;
        return Ok(());
    }
}

#[derive(Copy, Clone)]
pub struct SectionHeader
{
    pub pointer: u64, //+0
    pub csize: u32, //+8
    pub size: u32, //+12
    pub chksum: u32, //+16
    pub btype: u8, //+20
    pub flags: u8 //+21
}

impl SectionHeader
{
    pub fn read<TReader: io::Read>(reader: &mut TReader) -> io::Result<(u32, SectionHeader)>
    {
        let mut buf: [u8;SIZE_SECTION_HEADER] = [0;SIZE_SECTION_HEADER];
        let mut checksum: u32 = 0;

        reader.read(&mut buf)?;
        for i in 0..SIZE_SECTION_HEADER
        {
            checksum += buf[i] as u32;
        }
        return Ok((checksum, SectionHeader {
            pointer: LittleEndian::read_u64(&buf[0..8]),
            csize: LittleEndian::read_u32(&buf[8..12]),
            size: LittleEndian::read_u32(&buf[12..16]),
            chksum: LittleEndian::read_u32(&buf[16..20]),
            btype: buf[20],
            flags: buf[21]
        }));
    }

    pub fn new() -> SectionHeader
    {
        return SectionHeader
        {
            pointer: 0, //+0
            csize: 0, //+8
            size: 0, //+12
            chksum: 0, //+16
            btype: 0, //+20
            flags: FLAG_CHECK_WEAK // +21
        };
    }

    pub fn is_huge_section(&self) -> bool
    {
        return self.size > 100000000; //Return true if uncompressed size is greater than 100Mb
    }

    fn to_bytes(&self) -> [u8; SIZE_SECTION_HEADER]
    {
        let mut block: [u8; SIZE_SECTION_HEADER] = [0; SIZE_SECTION_HEADER];
        LittleEndian::write_u64(&mut block[0..8], self.pointer);
        LittleEndian::write_u32(&mut block[8..12], self.csize);
        LittleEndian::write_u32(&mut block[12..16], self.size);
        LittleEndian::write_u32(&mut block[16..20], self.chksum);
        block[20] = self.btype;
        block[21] = self.flags;
        return block;
    }

    pub fn get_checksum(&self) -> u32
    {
        let mut checksum: u32 = 0;
        let buf = self.to_bytes();
        for i in 0..SIZE_SECTION_HEADER
        {
            checksum += buf[i] as u32;
        }
        return checksum;
    }

    pub fn write<TWriter: io::Write>(&self, writer: &mut TWriter) -> io::Result<()>
    {
        let buf = self.to_bytes();
        writer.write(&buf)?;
        writer.flush()?;
        return Ok(());
    }
}
