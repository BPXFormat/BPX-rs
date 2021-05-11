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

use std::io::SeekFrom;
use std::string::String;
use std::path::Path;
use std::fs::DirEntry;
use std::collections::HashMap;

use crate::section::SectionData;
use crate::Interface;
use crate::SectionHandle;
use crate::Result;
use crate::error::Error;

pub struct StringSection
{
    handle: SectionHandle,
    cache: HashMap<u32, String>
}

impl StringSection
{
    pub fn new(hdl: SectionHandle) -> StringSection
    {
        return StringSection
        {
            handle: hdl,
            cache: HashMap::new()
        };
    }

    pub fn get(&mut self, interface: &mut dyn Interface, address: u32) -> Result<String>
    {
        if let Some(s) = self.cache.get(&address)
        {
            return Ok(s.clone());
        }
        let data = interface.open_section(self.handle)?;
        let s = low_level_read_string(address, data)?;
        self.cache.insert(address, s.clone());
        return Ok(s);
    }

    pub fn put(&mut self, interface: &mut dyn Interface, s: &str) -> Result<u32>
    {
        let data = interface.open_section(self.handle)?;
        let address = low_level_write_string(s, data)?;
        self.cache.insert(address, String::from(s));
        return Ok(address);
    }
}

fn low_level_read_string(ptr: u32, string_section: &mut dyn SectionData) -> Result<String>
{
    let mut curs: Vec<u8> = Vec::new();
    let mut chr: [u8; 1] = [0; 1]; //read char by char with a buffer

    string_section.seek(SeekFrom::Start(ptr as u64))?;
    string_section.read(&mut chr)?;
    while chr[0] != 0x0
    {
        curs.push(chr[0]);
        let res = string_section.read(&mut chr)?;
        if res != 1
        {
            return Err(Error::Truncation("string secton read"));
        }
    }
    match String::from_utf8(curs)
    {
        Err(e) => return Err(Error::Utf8("string section read")),
        Ok(v) => return Ok(v)
    }
}

fn low_level_write_string(s: &str, string_section: &mut dyn SectionData) -> Result<u32>
{
    let ptr = string_section.size() as u32;
    string_section.write(s.as_bytes())?;
    string_section.write(&[0x0])?;
    return Ok(ptr);
}

pub fn get_name_from_path(path: &Path) -> Result<String>
{
    match path.file_name()
    {
        Some(v) => match v.to_str()
        {
            Some(v) => return Ok(String::from(v)),
            // Panic here as a non Unicode system in all cases could just throw a bunch of broken unicode strings in a BPXP
            // The reason BPXP cannot support non-unicode strings in paths is simply because this would be incompatible with unicode systems
            None => panic!("Non unicode paths operating systems cannot run BPXP")
        },
        None => return Err(Error::from("incorrect path format")),
    }
}

pub fn get_name_from_dir_entry(entry: &DirEntry) -> String //Rust wants reallocation and slow code then give it fuck at the end
{
    match entry.file_name().to_str()
    {
        Some(v) => return String::from(v),
        None => panic!("Non unicode paths operating systems cannot run BPXP")
    }
}
