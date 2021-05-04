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
use xz::stream::Stream;
use std::num::Wrapping;

pub const SIZE_SECTION_HEADER: usize = 24;

#[derive(Copy, Clone)]
pub struct BPXSectionHeader
{
    pub pointer: u64, //+0
    pub csize: u32, //+8
    pub size: u32, //+12
    pub chksum: u32, //+16
    pub btype: u8, //+20
    pub flags: u8 //+21
}

impl BPXSectionHeader
{
    pub fn read<TReader: io::Read>(reader: &mut TReader) -> io::Result<(u32, BPXSectionHeader)>
    {
        let mut buf: [u8;SIZE_SECTION_HEADER] = [0;SIZE_SECTION_HEADER];
        let mut checksum: u32 = 0;

        reader.read(&mut buf)?;
        for i in 0..SIZE_SECTION_HEADER
        {
            checksum += buf[i] as u32;
        }
        return Ok((checksum, BPXSectionHeader {
            pointer: LittleEndian::read_u64(&buf[0..8]),
            csize: LittleEndian::read_u32(&buf[8..12]),
            size: LittleEndian::read_u32(&buf[12..16]),
            chksum: LittleEndian::read_u32(&buf[16..20]),
            btype: buf[20],
            flags: buf[21]
        }));
    }

    pub fn new(size: u32, btype: u8) -> BPXSectionHeader
    {
        return BPXSectionHeader
        {
            pointer: 0, //+0
            csize: 0, //+8
            size: size, //+12
            chksum: 0, //+16
            btype: btype, //+20
            flags: FLAG_CHECK_WEAK // +21
        };
    }

    pub fn is_huge_section(&self) -> bool
    {
        return self.size > 1000000; //Return true if uncompressed size is greater than 1Mb
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

pub trait Section : io::Read + io::Write + io::Seek
{
    fn load_in_memory(&mut self) -> io::Result<Vec<u8>>;
    fn size(&self) -> usize; //The computed size of the section
}

struct InMemorySection
{
    data: Vec<u8>,
    cursor: usize,
    cur_size: usize
}

impl InMemorySection
{
    pub fn new(data: Vec<u8>) -> InMemorySection
    {
        return InMemorySection
        {
            data: data,
            cursor: 0,
            cur_size: 0
        }
    }
}

impl io::Read for InMemorySection
{
    fn read(&mut self, data: &mut [u8]) -> io::Result<usize>
    {
        for i in 0..data.len()
        {
            if self.cursor >= self.data.len()
            {
                return Ok(i);
            }
            data[i] = self.data[self.cursor];
            self.cursor += 1;
        }
        return Ok(data.len())
    }
}

impl io::Write for InMemorySection
{
    fn write(&mut self, data: &[u8]) -> io::Result<usize>
    {
        for i in 0..data.len()
        {
            if self.cursor >= self.data.len()
            {
                return Ok(i);
            }
            self.data[self.cursor] = data[i];
            self.cursor += 1;
            if self.cursor >= self.cur_size
            {
                self.cur_size += 1
            }
        }
        return Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()>
    {
        return Ok(());
    }
}

fn slow_but_correct_add(value: usize, offset: isize) -> usize
//Unfortunatly rust requires much slower add operation (another reason to not use rust for large scale projects)
{
    if offset < 0
    {
        return value - -offset as usize;
    }
    else if offset > 0
    {
        return value + offset as usize;
    }
    else
    {
        return value;
    }
}

impl io::Seek for InMemorySection
{
    fn seek(&mut self, state: io::SeekFrom) -> io::Result<u64>
    {
        match state
        {
            io::SeekFrom::Start(pos) => self.cursor = pos as usize,
            io::SeekFrom::End(pos) => self.cursor = slow_but_correct_add(self.data.len(), pos as isize),
            io::SeekFrom::Current(pos) => self.cursor = slow_but_correct_add(self.cursor, pos as isize)
        }
        return Ok(self.cursor as u64);
    }
}

impl Section for InMemorySection
{
    fn load_in_memory(&mut self) -> io::Result<Vec<u8>>
    {
        return Ok(self.data.clone());
    }

    fn size(&self) -> usize
    {
        return self.cur_size;
    }
}

const SMALL_READ_BLOCK_SIZE: usize = 8192;

struct FileBasedSection
{
    data: File,
    buffer: [u8; SMALL_READ_BLOCK_SIZE],
    written: usize,
    cursor: usize,
    cur_size: usize,
    seek_ptr: u64
}

impl FileBasedSection
{
    pub fn new(data: File) -> FileBasedSection
    {
        return FileBasedSection
        {
            data: data,
            buffer: [0; SMALL_READ_BLOCK_SIZE],
            written: 0,
            cursor: usize::MAX,
            cur_size: 0,
            seek_ptr: 0
        };
    }
}

impl io::Read for FileBasedSection
{
    fn read(&mut self, data: &mut [u8]) -> io::Result<usize>
    {
        let mut cnt: usize = 0;

        for i in 0..data.len()
        {
            if self.cursor >= self.written
            {
                self.cursor = 0;
                self.written = self.data.read(&mut self.buffer)?;
            }
            if self.cursor < self.written
            {
                data[i] = self.buffer[self.cursor];
                self.cursor += 1;
                cnt += 1;
            }
        }
        return Ok(cnt);
    }
}

impl io::Write for FileBasedSection
{
    fn write(&mut self, data: &[u8]) -> io::Result<usize>
    {
        let len = self.data.write(data)?;
        if self.seek_ptr >= self.cur_size as u64
        {
            self.cur_size += len;
            self.seek_ptr += len as u64;
        }
        return Ok(len);
    }

    fn flush(&mut self) -> io::Result<()>
    {
        self.data.seek(io::SeekFrom::Current(self.cursor as i64))?;
        self.cursor = usize::MAX;
        return self.data.flush();
    }
}

impl io::Seek for FileBasedSection
{
    fn seek(&mut self, state: io::SeekFrom) -> io::Result<u64>
    {
        self.seek_ptr = self.data.seek(state)?;
        return Ok(self.seek_ptr);
    }
}

impl Section for FileBasedSection
{
    fn load_in_memory(&mut self) -> io::Result<Vec<u8>>
    {
        let mut data: Vec<u8> = Vec::new();
        self.data.read_to_end(&mut data)?;
        return Ok(data);
    }

    fn size(&self) -> usize
    {
        return self.cur_size;
    }
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
const READ_BLOCK_SIZE: usize = 65536;

fn block_based_deflate(input: &mut dyn Read, output: &mut dyn Write, inflated_size: usize) -> io::Result<(usize, u32)>
{
    let mut count: usize = 0;
    let mut encoder = match Stream::new_easy_encoder(0, xz::stream::Check::None)
    {
        Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("[BPX] deflate initialization error: {}", e))),
        Ok(v) => v
    };
    let mut chksum: Wrapping<u32> = Wrapping(0);
    let mut csize: usize = 0;

    while count < inflated_size {
        let mut idata: [u8; READ_BLOCK_SIZE] = [0; READ_BLOCK_SIZE];
        let mut status = xz::stream::Status::Ok;
        let mut expected = xz::stream::Status::MemNeeded;
        let mut action = xz::stream::Action::Run;
        let mut res = input.read(&mut idata)?;
        count += res;
        chksum += read_chksum(&idata);
        if count >= inflated_size
        {
            action = xz::stream::Action::Finish;
            expected = xz::stream::Status::StreamEnd;
        }
        while status != expected
        {
            let mut odata: Vec<u8> = Vec::with_capacity(READ_BLOCK_SIZE * 2);
            match encoder.process_vec(&idata[0..res], &mut odata, action)
            {
                Ok(s) => status = s,
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("[BPX] deflate error: {}", e)))
            }
            res = 0;
            output.write(&odata)?;
            csize += odata.len();
        }
    }
    return Ok((csize, chksum.0));
}

fn block_based_inflate(input: &mut dyn Read, output: &mut dyn Write, deflated_size: usize) -> io::Result<u32>
{
    let mut decoder = match Stream::new_stream_decoder(u32::MAX as u64, xz::stream::CONCATENATED)
    {
        Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("[BPX] inflate error: {}", e))),
        Ok(v) => v
    };
    let mut action = xz::stream::Action::Run;
    let mut expected = xz::stream::Status::MemNeeded;
    let mut chksum: Wrapping<u32> = Wrapping(0);
    let mut remaining = deflated_size;

    while remaining > 0 {
        let mut idata: [u8; READ_BLOCK_SIZE] = [0; READ_BLOCK_SIZE];
        let mut status = xz::stream::Status::Ok;
        let mut res = input.read(&mut idata[0..std::cmp::min(READ_BLOCK_SIZE, remaining)])?;
        remaining -= res;
        if remaining == 0
        {
            action = xz::stream::Action::Finish;
            expected = xz::stream::Status::StreamEnd;
        }
        while status != expected
        {
            let mut odata: Vec<u8> = Vec::with_capacity(READ_BLOCK_SIZE * 16);
            match decoder.process_vec(&idata[0..res], &mut odata, action)
            {
                Ok(s) => status = s,
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("[BPX] inflate error: {}", e)))
            }
            res = 0;
            chksum += read_chksum(&odata);
            output.write(&odata)?;
        }
    }
    output.flush()?;
    return Ok(chksum.0);
}

fn load_section_in_memory(bpx: &mut File, header: &BPXSectionHeader) -> io::Result<InMemorySection>
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

fn load_section_as_file(bpx: &mut File, header: &BPXSectionHeader) -> io::Result<FileBasedSection>
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

pub fn open_section(bpx: &mut File, header: &BPXSectionHeader) -> io::Result<Box<dyn Section>>
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

pub fn create_section(header: &BPXSectionHeader) -> io::Result<Box<dyn Section>>
{
    if header.is_huge_section() || header.size == 0
    {
        let mut section = FileBasedSection::new(tempfile::tempfile()?);
        section.seek(io::SeekFrom::Start(0))?;
        return Ok(Box::from(section));
    }
    else
    {
        let mut section = InMemorySection::new(vec![0; header.size as usize]);
        section.seek(io::SeekFrom::Start(0))?;
        return Ok(Box::from(section));
    }
}

pub fn write_section(section: &mut Box<dyn Section>, out: &mut dyn Write) -> io::Result<(usize, u32, u8)>
{
    if section.size() < READ_BLOCK_SIZE
    {
        let mut idata: [u8; READ_BLOCK_SIZE] = [0; READ_BLOCK_SIZE];
        let mut count: usize = 0;
        let mut chksum: Wrapping<u32> = Wrapping(0);
        while count < section.size() as usize
        {
            let res = section.read(&mut idata)?;
            out.write(&idata[0..res])?;
            chksum += read_chksum(&idata[0..res]);
            count += res;
        }
        section.flush()?;
        return Ok((section.size(), chksum.0, FLAG_CHECK_WEAK));
    }
    else
    {
        let size = section.size();
        let (csize, chksum) = block_based_deflate(section, out, size)?;
        return Ok((csize, chksum, FLAG_CHECK_WEAK | FLAG_COMPRESS_XZ));
    }
}
