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

//! The BPX decoder.

use std::{io, io::Write, rc::Rc};

use crate::{
    compression::{Checksum, Crc32Checksum, Inflater, WeakChecksum, XzCompressionMethod, ZlibCompressionMethod},
    error::ReadError,
    header::{
        MainHeader,
        SectionHeader,
        Struct,
        FLAG_CHECK_CRC32,
        FLAG_CHECK_WEAK,
        FLAG_COMPRESS_XZ,
        FLAG_COMPRESS_ZLIB
    },
    section::AutoSection,
    utils::OptionExtension,
    Interface,
    SectionHandle
};

const READ_BLOCK_SIZE: usize = 8192;

/// Represents the IO backend for a BPX decoder.
pub trait IoBackend: io::Seek + io::Read
{
}
impl<T: io::Seek + io::Read> IoBackend for T {}

/// The BPX decoder.
pub struct Decoder<TBackend: IoBackend>
{
    main_header: MainHeader,
    sections: Vec<SectionHeader>,
    sections_data: Vec<Option<Rc<AutoSection>>>,
    file: TBackend
}

impl<TBackend: IoBackend> Decoder<TBackend>
{
    fn read_section_header_table(&mut self, checksum: u32) -> Result<(), ReadError>
    {
        let mut final_checksum = checksum;

        for _ in 0..self.main_header.section_num {
            let (checksum, header) = SectionHeader::read(&mut self.file)?;
            final_checksum += checksum;
            self.sections.push(header);
        }
        if final_checksum != self.main_header.chksum {
            return Err(ReadError::Checksum(final_checksum, self.main_header.chksum));
        }
        return Ok(());
    }

    /// Creates a new BPX decoder.
    ///
    /// # Arguments
    ///
    /// * `file`: An [IoBackend](self::IoBackend) to use for reading the data.
    ///
    /// returns: Result<Decoder<TBackend>, Error>
    ///
    /// # Errors
    ///
    /// An [Error](crate::error::Error) is returned if some headers
    /// could not be read or if the header data is corrupted.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Seek, SeekFrom};
    /// use bpx::decoder::Decoder;
    /// use bpx::encoder::Encoder;
    /// use bpx::Interface;
    /// use bpx::utils::new_byte_buf;
    ///
    /// let mut encoder = Encoder::new(new_byte_buf(0)).unwrap();
    /// encoder.save();
    /// let mut bytebuf = encoder.into_inner();
    /// bytebuf.seek(SeekFrom::Start(0)).unwrap();
    /// let mut decoder = Decoder::new(bytebuf).unwrap();
    /// assert_eq!(decoder.get_main_header().section_num, 0);
    /// assert_eq!(decoder.get_main_header().btype, 'P' as u8);
    /// ```
    pub fn new(mut file: TBackend) -> Result<Decoder<TBackend>, ReadError>
    {
        let (checksum, header) = MainHeader::read(&mut file)?;
        let num = header.section_num;
        let mut decoder = Decoder {
            file,
            main_header: header,
            sections: Vec::with_capacity(num as usize),
            sections_data: std::iter::repeat_with(|| None).take(num as usize).collect()
        };
        decoder.read_section_header_table(checksum)?;
        return Ok(decoder);
    }

    pub fn load_section(&mut self, handle: SectionHandle) -> Result<&Rc<AutoSection>, ReadError>
    {
        let header = &self.sections[handle.0];
        let file = &mut self.file;
        let object = self.sections_data[handle.0].get_or_insert_with_err(|| load_section(file, handle, header))?;
        return Ok(object);
    }

    /// Consumes this BPX decoder and returns the inner IO backend.
    pub fn into_inner(self) -> TBackend
    {
        return self.file;
    }
}

impl<TBackend: IoBackend> Interface for Decoder<TBackend>
{
    fn find_section_by_type(&self, btype: u8) -> Option<SectionHandle>
    {
        for i in 0..self.sections.len() {
            if self.sections[i].btype == btype {
                return Some(SectionHandle(i));
            }
        }
        return None;
    }

    fn find_all_sections_of_type(&self, btype: u8) -> Vec<SectionHandle>
    {
        let mut v = Vec::new();

        for i in 0..self.sections.len() {
            if self.sections[i].btype == btype {
                v.push(SectionHandle(i));
            }
        }
        return v;
    }

    fn find_section_by_index(&self, index: u32) -> Option<SectionHandle>
    {
        if let Some(_) = self.sections.get(index as usize) {
            return Some(SectionHandle(index as _));
        }
        return None;
    }

    fn get_section_header(&self, handle: SectionHandle) -> &SectionHeader
    {
        return &self.sections[handle.0];
    }

    fn get_section_index(&self, handle: SectionHandle) -> u32
    {
        return handle.0 as u32;
    }

    fn get_section(&self, handle: SectionHandle) -> &Rc<AutoSection>
    {
        return self.sections_data[handle.0].as_ref().unwrap();
    }

    fn get_main_header(&self) -> &MainHeader
    {
        return &self.main_header;
    }
}

fn load_section<TBackend: IoBackend>(
    file: &mut TBackend,
    handle: SectionHandle,
    section: &SectionHeader
) -> Result<Rc<AutoSection>, ReadError>
{
    let sdata = Rc::new(AutoSection::new(section.size, handle)?);
    {
        let mut data = sdata.open().unwrap();
        data.seek(io::SeekFrom::Start(0))?;
        if section.flags & FLAG_CHECK_WEAK != 0 {
            let mut chksum = WeakChecksum::new();
            //TODO: Check
            load_section_checked(file, &section, data.as_mut(), &mut chksum)?;
            let v = chksum.finish();
            if v != section.chksum {
                return Err(ReadError::Checksum(v, section.chksum));
            }
        } else if section.flags & FLAG_CHECK_CRC32 != 0 {
            let mut chksum = Crc32Checksum::new();
            //TODO: Check
            load_section_checked(file, &section, data.as_mut(), &mut chksum)?;
            let v = chksum.finish();
            if v != section.chksum {
                return Err(ReadError::Checksum(v, section.chksum));
            }
        } else {
            let mut chksum = WeakChecksum::new();
            //TODO: Check
            load_section_checked(file, &section, data.as_mut(), &mut chksum)?;
        }
        data.seek(io::SeekFrom::Start(0))?;
    } //Amazing: another defect of the Rust borrow checker still so stupid
    return Ok(sdata);
}

fn load_section_checked<TBackend: io::Read + io::Seek, TWrite: Write, TChecksum: Checksum>(
    file: &mut TBackend,
    section: &SectionHeader,
    out: &mut TWrite,
    chksum: &mut TChecksum
) -> Result<(), ReadError>
{
    if section.flags & FLAG_COMPRESS_XZ != 0 {
        load_section_compressed::<XzCompressionMethod, _, _, _>(file, &section, out, chksum)?;
    } else if section.flags & FLAG_COMPRESS_ZLIB != 0 {
        load_section_compressed::<ZlibCompressionMethod, _, _, _>(file, &section, out, chksum)?;
    } else {
        load_section_uncompressed(file, &section, out, chksum)?;
    }
    return Ok(());
}

fn load_section_uncompressed<TBackend: io::Read + io::Seek, TWrite: Write, TChecksum: Checksum>(
    bpx: &mut TBackend,
    header: &SectionHeader,
    output: &mut TWrite,
    chksum: &mut TChecksum
) -> io::Result<()>
{
    let mut idata: [u8; READ_BLOCK_SIZE] = [0; READ_BLOCK_SIZE];
    let mut count: usize = 0;
    let mut remaining: usize = header.size as usize;

    bpx.seek(io::SeekFrom::Start(header.pointer))?;
    while count < header.size as usize {
        let res = bpx.read(&mut idata[0..std::cmp::min(READ_BLOCK_SIZE, remaining)])?;
        output.write(&idata[0..res])?;
        chksum.push(&idata[0..res]);
        count += res;
        remaining -= res;
    }
    return Ok(());
}

fn load_section_compressed<TMethod: Inflater, TBackend: io::Read + io::Seek, TWrite: Write, TChecksum: Checksum>(
    bpx: &mut TBackend,
    header: &SectionHeader,
    output: &mut TWrite,
    chksum: &mut TChecksum
) -> Result<(), ReadError>
{
    bpx.seek(io::SeekFrom::Start(header.pointer))?;
    XzCompressionMethod::inflate(bpx, output, header.csize as usize, chksum)?;
    return Ok(());
}
