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

use std::path::PathBuf;
use std::path::Path;
use std::io::Read;
use std::fs::File;
use std::io;
use std::io::Write;
use byteorder::LittleEndian;
use byteorder::ByteOrder;

use crate::decoder::IoBackend;
use crate::decoder::Decoder;
use crate::Interface;
use crate::Result;
use crate::error::Error;
use crate::bpxp::Architecture;
use crate::bpxp::Platform;
use crate::SectionHandle;
use crate::header::SECTION_TYPE_STRING;
use crate::header::SECTION_TYPE_SD;
use crate::sd::Object;
use crate::bpxp::DATA_SECTION_TYPE;
use crate::strings::StringSection;

const DATA_READ_BUFFER_SIZE: usize = 8192;

pub struct PackageDecoder
{
    type_code: [u8; 2],
    architecture: Architecture,
    platform: Platform,
    strings: SectionHandle
}

fn get_arch_platform_from_code(acode: u8, pcode: u8) -> Result<(Architecture, Platform)>
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
        _ => return Err(Error::Corruption(String::from("Architecture code does not exist")))
    }
    match pcode
    {
        0x0 => platform = Platform::Linux,
        0x1 => platform = Platform::Mac,
        0x2 => platform = Platform::Windows,
        0x3 => platform = Platform::Android,
        0x4 => platform = Platform::Any,
        _ => return Err(Error::Corruption(String::from("Platform code does not exist")))
    }
    return Ok((arch, platform));
}

impl PackageDecoder
{
    pub fn read<TBackend: IoBackend>(decoder: &mut Decoder<TBackend>) -> Result<PackageDecoder>
    {
        if decoder.get_main_header().btype != 'P' as u8
        {
            return Err(Error::Corruption(format!("Unknown type of BPX: {}", decoder.get_main_header().btype as char)));
        }
        let (a, p) = get_arch_platform_from_code(decoder.get_main_header().type_ext[0], decoder.get_main_header().type_ext[1])?;
        let strings = match decoder.find_section_by_type(SECTION_TYPE_STRING)
        {
            Some(v) => v,
            None => return Err(Error::Corruption(String::from("Unable to locate strings section")))
        };
        return Ok(PackageDecoder
        {
            architecture: a,
            platform: p,
            strings: strings,
            type_code: [decoder.get_main_header().type_ext[2], decoder.get_main_header().type_ext[3]]
        });
    }

    pub fn get_variant(&self) -> [u8; 2]
    {
        return self.type_code;
    }

    pub fn get_architecture(&self) -> Architecture
    {
        return self.architecture;
    }

    pub fn get_platform(&self) -> Platform
    {
        return self.platform;
    }

    pub fn read_metadata<TBackend: IoBackend>(&self, decoder: &mut Decoder<TBackend>) -> Result<Option<Object>>
    {
        if let Some(handle) = decoder.find_section_by_type(SECTION_TYPE_SD)
        {
            let mut data = decoder.open_section(handle)?;
            let obj = Object::read(&mut data)?;
            return Ok(Some(obj));
        }
        return Ok(None);
    }

    fn extract_file(&self, source: &mut dyn Read, dest: &PathBuf, size: u64) -> io::Result<Option<(u64, File)>>
    {
        if let Some(v) = dest.parent()
        {
            std::fs::create_dir_all(v)?;
        }
        let mut fle = File::create(dest)?;
        let mut v: Vec<u8> = Vec::with_capacity(DATA_READ_BUFFER_SIZE);
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
            if v.len() >= DATA_READ_BUFFER_SIZE
            {
                fle.write(&v)?;
                v = Vec::with_capacity(DATA_READ_BUFFER_SIZE);
            }
            count += 1;
        }
        fle.write(&v)?;
        return Ok(None);
    }

    fn continue_file(&self, source: &mut dyn Read, out: &mut dyn Write, size: u64) -> io::Result<u64>
    {
        let mut v: Vec<u8> = Vec::with_capacity(DATA_READ_BUFFER_SIZE);
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
            if v.len() >= DATA_READ_BUFFER_SIZE
            {
                out.write(&v)?;
                v = Vec::with_capacity(DATA_READ_BUFFER_SIZE);
            }
            count += 1;
        }
        return Ok(0);
    }

    pub fn unpack<TBackend: IoBackend>(&self, decoder: &mut Decoder<TBackend>, target: &Path) -> Result<()>
    {
        let mut strings = StringSection::new(self.strings);
        let secs = decoder.find_all_sections_of_type(DATA_SECTION_TYPE);
        let mut truncated: Option<(u64, File)> = None;
        for v in secs
        {
            let header = *decoder.get_section_header(v);
            if let Some((remaining, mut file)) = std::mem::replace(&mut truncated, None)
            {
                let mut section = decoder.open_section(v)?;
                let res = self.continue_file(&mut section, &mut file, remaining)?;
                if res > 0 //Still not finished
                {
                    truncated = Some((res, file));
                    continue;
                }
            }
            let mut count: u64 = 0;
            while count < header.size as u64
            {
                let mut fheader: [u8; 12] = [0; 12];
                {
                    let section = decoder.open_section(v)?;
                    section.read(&mut fheader)?;
                }
                let path = strings.get(decoder, LittleEndian::read_u32(&fheader[8..12]))?;
                if path == ""
                {
                    return Err(Error::Corruption(String::from("Empty path string detected, aborting to prevent damage on host files")));
                }
                let size = LittleEndian::read_u64(&fheader[0..8]);
                println!("Reading {} with {} byte(s)...", path, size);
                let mut dest = PathBuf::new();
                dest.push(target);
                dest.push(path);
                {
                    let mut section = decoder.open_section(v)?;
                    truncated = self.extract_file(&mut section, &dest, size)?;
                }
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
