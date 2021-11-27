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
use crate::compression::{Checksum, Crc32Checksum, Inflater, WeakChecksum, XzCompressionMethod, ZlibCompressionMethod};
use crate::core::header::{FLAG_CHECK_CRC32, FLAG_CHECK_WEAK, FLAG_COMPRESS_XZ, FLAG_COMPRESS_ZLIB, SectionHeader};
use crate::error::ReadError;

use crate::section::{new_section_data, SectionData};
use crate::utils::ReadFill;
//use crate::section::{new_section_data, SectionData};

const READ_BLOCK_SIZE: usize = 8192;

/*/// Represents the IO backend for a BPX decoder.
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
        Ok(())
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
    /// A [ReadError](crate::error::ReadError) is returned if some headers
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
        Ok(decoder)
    }

    /// Loads a section from this BPX.
    ///
    /// # Arguments
    ///
    /// * `handle`: a handle to the section.
    ///
    /// returns: Result<&Rc<AutoSection>, ReadError>
    ///
    /// # Errors
    ///
    /// A [ReadError](crate::error::ReadError) is returned if the section could not be loaded.
    pub fn load_section(&mut self, handle: Handle) -> Result<&Rc<AutoSection>, ReadError>
    {
        let header = &self.sections[handle.0 as usize];
        let file = &mut self.file;
        let object = self.sections_data[handle.0 as usize]
            .get_or_insert_with_err(|| load_section(file, handle, header))?;
        Ok(object)
    }

    /// Consumes this BPX decoder and returns the inner IO backend.
    pub fn into_inner(self) -> TBackend
    {
        self.file
    }
}

impl<TBackend: IoBackend> Interface for Decoder<TBackend>
{
    fn find_section_by_type(&self, btype: u8) -> Option<Handle>
    {
        for i in 0..self.sections.len() {
            if self.sections[i].btype == btype {
                return Some(Handle(i as u32));
            }
        }
        None
    }

    fn find_all_sections_of_type(&self, btype: u8) -> Vec<Handle>
    {
        let mut v = Vec::new();

        for i in 0..self.sections.len() {
            if self.sections[i].btype == btype {
                v.push(Handle(i as u32));
            }
        }
        v
    }

    fn find_section_by_index(&self, index: u32) -> Option<Handle>
    {
        if self.sections.get(index as usize).is_some() {
            return Some(Handle(index as _));
        }
        None
    }

    fn get_section_header(&self, handle: Handle) -> &SectionHeader
    {
        &self.sections[handle.0 as usize]
    }

    fn get_section_index(&self, handle: Handle) -> u32
    {
        handle.0 as u32
    }

    fn get_section(&self, handle: Handle) -> &Rc<AutoSection>
    {
        return self.sections_data[handle.0 as usize].as_ref().unwrap();
    }

    fn get_main_header(&self) -> &MainHeader
    {
        &self.main_header
    }
}*/

/*fn load_section<TBackend: IoBackend>(
    file: &mut TBackend,
    handle: Handle,
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
            load_section_checked(file, section, data.as_mut(), &mut chksum)?;
            let v = chksum.finish();
            if v != section.chksum {
                return Err(ReadError::Checksum(v, section.chksum));
            }
        } else if section.flags & FLAG_CHECK_CRC32 != 0 {
            let mut chksum = Crc32Checksum::new();
            //TODO: Check
            load_section_checked(file, section, data.as_mut(), &mut chksum)?;
            let v = chksum.finish();
            if v != section.chksum {
                return Err(ReadError::Checksum(v, section.chksum));
            }
        } else {
            let mut chksum = WeakChecksum::new();
            //TODO: Check
            load_section_checked(file, section, data.as_mut(), &mut chksum)?;
        }
        data.seek(io::SeekFrom::Start(0))?;
    } //Amazing: another defect of the Rust borrow checker still so stupid
    Ok(sdata)
}*/

pub fn load_section1<T: io::Read + io::Seek>(
    file: &mut T,
    section: &SectionHeader
) -> Result<Box<dyn SectionData>, ReadError>
{
    let mut data = new_section_data(Some(section.size))?;
    data.seek(io::SeekFrom::Start(0))?;
    if section.flags & FLAG_CHECK_WEAK != 0 {
        let mut chksum = WeakChecksum::new();
        //TODO: Check
        load_section_checked(file, section, data.as_mut(), &mut chksum)?;
        let v = chksum.finish();
        if v != section.chksum {
            return Err(ReadError::Checksum(v, section.chksum));
        }
    } else if section.flags & FLAG_CHECK_CRC32 != 0 {
        let mut chksum = Crc32Checksum::new();
        //TODO: Check
        load_section_checked(file, section, data.as_mut(), &mut chksum)?;
        let v = chksum.finish();
        if v != section.chksum {
            return Err(ReadError::Checksum(v, section.chksum));
        }
    } else {
        let mut chksum = WeakChecksum::new();
        //TODO: Check
        load_section_checked(file, section, data.as_mut(), &mut chksum)?;
    }
    data.seek(io::SeekFrom::Start(0))?;
    Ok(data)
}

fn load_section_checked<TBackend: io::Read + io::Seek, TWrite: Write, TChecksum: Checksum>(
    file: &mut TBackend,
    section: &SectionHeader,
    out: TWrite,
    chksum: &mut TChecksum
) -> Result<(), ReadError>
{
    if section.flags & FLAG_COMPRESS_XZ != 0 {
        load_section_compressed::<XzCompressionMethod, _, _, _>(file, section, out, chksum)?;
    } else if section.flags & FLAG_COMPRESS_ZLIB != 0 {
        load_section_compressed::<ZlibCompressionMethod, _, _, _>(file, section, out, chksum)?;
    } else {
        load_section_uncompressed(file, section, out, chksum)?;
    }
    Ok(())
}

fn load_section_uncompressed<TBackend: io::Read + io::Seek, TWrite: Write, TChecksum: Checksum>(
    bpx: &mut TBackend,
    header: &SectionHeader,
    mut output: TWrite,
    chksum: &mut TChecksum
) -> io::Result<()>
{
    let mut idata: [u8; READ_BLOCK_SIZE] = [0; READ_BLOCK_SIZE];
    let mut count: usize = 0;
    let mut remaining: usize = header.size as usize;

    bpx.seek(io::SeekFrom::Start(header.pointer))?;
    while count < header.size as usize {
        let res = bpx.read_fill(&mut idata[0..std::cmp::min(READ_BLOCK_SIZE, remaining)])?;
        output.write_all(&idata[0..res])?;
        chksum.push(&idata[0..res]);
        count += res;
        remaining -= res;
    }
    Ok(())
}

fn load_section_compressed<
    TMethod: Inflater,
    TBackend: io::Read + io::Seek,
    TWrite: Write,
    TChecksum: Checksum
>(
    bpx: &mut TBackend,
    header: &SectionHeader,
    output: TWrite,
    chksum: &mut TChecksum
) -> Result<(), ReadError>
{
    bpx.seek(io::SeekFrom::Start(header.pointer))?;
    XzCompressionMethod::inflate(bpx, output, header.csize as usize, chksum)?;
    Ok(())
}
