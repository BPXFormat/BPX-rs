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
use std::path::PathBuf;
use std::vec::Vec;
use std::io;
use std::io::Write;
use std::io::Read;
use byteorder::LittleEndian;
use byteorder::ByteOrder;
use super::strings::*;
use std::fs::metadata;
use std::fs::read_dir;
use super::bpx;
use super::sd::Object;
use super::sd::load_structured_data;
use super::sd::write_structured_data;

const DATA_SECTION_TYPE: u8 = 0x1;

const DATA_WRITE_BUFFER_SIZE: usize = 8192;
const MIN_DATA_REMAINING_SIZE: usize = DATA_WRITE_BUFFER_SIZE;
const MAX_DATA_SECTION_SIZE: usize = 200000000 - MIN_DATA_REMAINING_SIZE; //200MB

pub enum Architecture
{
    X86_64,
    Aarch64,
    X86,
    Armv7hl,
    Any
}

pub enum Platform
{
    Linux,
    Mac,
    Windows,
    Android,
    Any
}

pub struct Decoder
{
    pub architecture: Architecture,
    pub platform: Platform,
    decoder: bpx::Decoder
}

fn get_arch_platform_from_code(acode: u8, pcode: u8) -> io::Result<(Architecture, Platform)>
{
    let arch;
    let platform;

    match acode
    {
        0x0 => arch = Architecture::X86_64,
        0x1 => arch = Architecture::Aarch64,
        0x2 => arch = Architecture::X86,
        0x3 => arch = Architecture::Armv7hl,
        0x4 => arch = Architecture::Any,
        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "[BPX] Architecture code does not exist"))
    }
    match pcode
    {
        0x0 => platform = Platform::Linux,
        0x1 => platform = Platform::Mac,
        0x2 => platform = Platform::Windows,
        0x3 => platform = Platform::Android,
        0x4 => platform = Platform::Any,
        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "[BPX] Platform code does not exist"))
    }
    return Ok((arch, platform));
}

impl Decoder
{
    pub fn new(file: &Path) -> io::Result<Decoder>
    {
        let decoder = bpx::Decoder::new(file)?;
        if decoder.main_header.btype != 'P' as u8
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("[BPX] Unknown type of BPX: {}", decoder.main_header.btype as char)));
        }
        let (a, p) = get_arch_platform_from_code(decoder.main_header.type_ext[0], decoder.main_header.type_ext[1])?;
        if decoder.main_header.type_ext[2] != 0x50 || decoder.main_header.type_ext[3] != 0x4B
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("[BPX] Unsupported BPXP variant {}{}", decoder.main_header.type_ext[2] as char, decoder.main_header.type_ext[3] as char)));
        }
        return Ok(Decoder
        {
            architecture: a,
            platform: p,
            decoder: decoder
        })
    }

    pub fn open_metadata(&mut self) -> io::Result<Object>
    {
        if let Some(section) = self.decoder.find_section_by_type(254)
        {
            let mut data = self.decoder.open_section(&section)?;
            return load_structured_data(&mut data);
        }
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "[BPX] could not locate metadata section"));
    }

    fn extract_file(&self, source: &mut dyn Read, dest: &PathBuf, size: u64) -> io::Result<Option<(u64, File)>>
    {
        if let Some(v) = dest.parent()
        {
            std::fs::create_dir_all(v)?;
        }
        let mut fle = File::create(dest)?;
        let mut v: Vec<u8> = Vec::with_capacity(DATA_WRITE_BUFFER_SIZE);
        let mut count: u64 = 0;
        while count < size
        {
            let mut byte: [u8; 1] = [0; 1];
            if source.read(&mut byte)? == 0 && count < size
            { //Well the file is divided in multiple sections signal the caller of the problen
                fle.write(&v)?;
                return Ok(Some((size - count, fle)));
            }
            v.push(byte[0]);
            if v.len() >= DATA_WRITE_BUFFER_SIZE
            {
                fle.write(&v)?;
                v = Vec::with_capacity(DATA_WRITE_BUFFER_SIZE);
            }
            count += 1;
        }
        fle.write(&v)?;
        return Ok(None);
    }

    fn continue_file(&self, source: &mut dyn Read, out: &mut dyn Write, size: u64) -> io::Result<u64>
    {
        let mut v: Vec<u8> = Vec::with_capacity(DATA_WRITE_BUFFER_SIZE);
        let mut count: u64 = 0;
        while count < size
        {
            let mut byte: [u8; 1] = [0; 1];
            if source.read(&mut byte)? == 0 && count < size
            { //Well the file is divided in multiple sections signal the caller of the problen
                out.write(&v)?;
                return Ok(size - count);
            }
            v.push(byte[0]);
            if v.len() >= DATA_WRITE_BUFFER_SIZE
            {
                out.write(&v)?;
                v = Vec::with_capacity(DATA_WRITE_BUFFER_SIZE);
            }
            count += 1;
        }
        return Ok(0);
    }

    pub fn unpack(&mut self, target: &Path) -> io::Result<()>
    {
        let mut strings = self.decoder.load_string_section()?;
        let secs = self.decoder.find_all_sections_of_type(DATA_SECTION_TYPE);
        let mut truncated: Option<(u64, File)> = None;
        for v in secs
        {
            let mut section = self.decoder.open_section(&v)?;
            if let Some((remaining, mut file)) = std::mem::replace(&mut truncated, None)
            {
                let res = self.continue_file(&mut section, &mut file, remaining)?;
                if res > 0 //Still not finished
                {
                    truncated = Some((res, file));
                    continue;
                }
            }
            let mut count: u64 = 0;
            while count < v.size as u64
            {
                let mut header: [u8; 12] = [0; 12];
                section.read(&mut header)?;
                let path = get_string(LittleEndian::read_u32(&header[8..12]), &mut strings)?;
                if path == ""
                {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "[BPX] Empty path string detected, aborting to prevent damage on host files"));
                }
                let size = LittleEndian::read_u64(&header[0..8]);
                println!("Reading {} with {} byte(s)...", path, size);
                let mut dest = PathBuf::new();
                dest.push(target);
                dest.push(path);
                truncated = self.extract_file(&mut section, &dest, size)?;
                if truncated.is_some()
                {
                    break;
                }
                count += size + 12;
            }
        }
        return Ok(());
    }
}

pub struct Encoder
{
    pub architecture: Architecture,
    pub platform: Platform,
    encoder: bpx::Encoder
}

impl Encoder
{
    pub fn new(file: &Path) -> io::Result<Encoder>
    {
        let encoder = bpx::Encoder::new(file)?;

        return Ok(Encoder
        {
            architecture: Architecture::Any,
            platform: Platform::Any,
            encoder: encoder
        });
    }

    fn write_file(&mut self, source: &mut dyn Read, data_id: usize) -> io::Result<bool>
    {
        let data = self.encoder.get_section_by_index(data_id);
        let mut buf: [u8; DATA_WRITE_BUFFER_SIZE] = [0; DATA_WRITE_BUFFER_SIZE];
        let mut res = source.read(&mut buf)?;

        while res > 0
        {
            data.write(&buf[0..res])?;
            if data.size() >= MAX_DATA_SECTION_SIZE //Split sections (this is to avoid reaching the 4Gb max)
            {
                return Ok(false);
            }
            res = source.read(&mut buf)?;
        }
        return Ok(true);
    }

    fn pack_file(&mut self, source: &Path, name: String, data_id1: usize, strings_id: usize) -> io::Result<usize>
    {
        let mut data_id = data_id1;
        let strings = self.encoder.get_section_by_index(strings_id);
        let size = metadata(source)?.len();
        let mut fle = File::open(source)?;
        let mut buf: [u8; 12] = [0; 12];

        println!("Writing file {} with {} byte(s)", name, size);
        LittleEndian::write_u64(&mut buf[0..8], size);
        LittleEndian::write_u32(&mut buf[8..12], write_string(&name, strings)?);
        {
            let data = self.encoder.get_section_by_index(data_id);
            data.write(&buf)?;
        }
        while !self.write_file(&mut fle, data_id)?
        {
            data_id = self.encoder.add_section(DATA_SECTION_TYPE, 0)?;
        }
        return Ok(data_id);
    }

    fn pack_dir(&mut self, source: &Path, name: String, data_id1: usize, strings_id: usize) -> io::Result<()>
    {
        let mut data_id = data_id1;
        let entries = read_dir(source)?;
    
        for rentry in entries
        {
            let entry = rentry?;
            let mut s = name.clone();
            s.push('/');
            s.push_str(&get_name_from_dir_entry(&entry));
            if entry.file_type()?.is_dir()
            {
                self.pack_dir(&entry.path(), s, data_id, strings_id)?
            }
            else
            {
                data_id = self.pack_file(&entry.path(), s, data_id, strings_id)?;
            }
        }
        return Ok(());
    }

    pub fn pack_vname(&mut self, source: &Path, vname: &str) -> io::Result<()>
    {
        let strings = match self.encoder.find_section_by_type(bpx::STRING_SECTION_TYPE)
        {
            Some(v) => v,
            None => self.encoder.add_section(bpx::STRING_SECTION_TYPE, 0)?
        };
        let md = metadata(source)?;
        let data_section = self.encoder.add_section(DATA_SECTION_TYPE, 0)?;
        if md.is_file()
        {
            self.pack_file(source, String::from(vname), data_section, strings)?;
            return Ok(());
        }
        else
        {
            return self.pack_dir(source, String::from(vname), data_section, strings);
        }
    }
    
    pub fn pack(&mut self, source: &Path) -> io::Result<()>
    {
        let strings = match self.encoder.find_section_by_type(bpx::STRING_SECTION_TYPE)
        {
            Some(v) => v,
            None => self.encoder.add_section(bpx::STRING_SECTION_TYPE, 0)?
        };
        let md = metadata(source)?;
        let data_section = self.encoder.add_section(DATA_SECTION_TYPE, 0)?;
        if md.is_file()
        {
            self.pack_file(source, get_name_from_path(source)?, data_section, strings)?;
            return Ok(());
        }
        else
        {
            return self.pack_dir(source, get_name_from_path(source)?, data_section, strings);
        }
    }

    pub fn add_metadata(&mut self, obj: &Object) -> io::Result<()>
    {
        let section = self.encoder.add_section(254, 0)?;
        let mut data = self.encoder.get_section_by_index(section);

        write_structured_data(&mut data, obj)?;
        return Ok(());
    }

    pub fn save(&mut self) -> io::Result<()>
    {
        match self.architecture
        {
            Architecture::X86_64 => self.encoder.main_header.type_ext[0] = 0x0,
            Architecture::Aarch64 => self.encoder.main_header.type_ext[0] = 0x1,
            Architecture::X86 => self.encoder.main_header.type_ext[0] = 0x2,
            Architecture::Armv7hl => self.encoder.main_header.type_ext[0] = 0x3,
            Architecture::Any => self.encoder.main_header.type_ext[0] = 0x4,
        }
        match self.platform
        {
            Platform::Linux => self.encoder.main_header.type_ext[1] = 0x0,
            Platform::Mac => self.encoder.main_header.type_ext[1] = 0x1,
            Platform::Windows => self.encoder.main_header.type_ext[1] = 0x2,
            Platform::Android => self.encoder.main_header.type_ext[1] = 0x3,
            Platform::Any => self.encoder.main_header.type_ext[1] = 0x4,
        }
        self.encoder.main_header.type_ext[2] = 0x50;
        self.encoder.main_header.type_ext[3] = 0x4B;
        return self.encoder.save();
    }
}
