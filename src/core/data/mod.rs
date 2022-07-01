// Copyright (c) 2022, BlockProject 3D
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

//! Utilities to manipulate the content of sections.

mod auto;
mod file;
mod memory;

use std::{
    io::{Read, Result, Seek, Write},
    vec::Vec,
};
use std::io::SeekFrom;

#[cfg(not(test))]
const SHIFT_BUF_SIZE: usize = 8192;
#[cfg(test)]
const SHIFT_BUF_SIZE: usize = 4;

/// Shift direction and amount for shifting section data.
pub enum Shift {
    /// Shift to the left.
    Left(u32),

    /// Shift to the right.
    Right(u32)
}

/// Write mode for [SectionData](crate::core::data::SectionData).
pub enum WriteMode {
    /// Write data by appending bytes at the current cursor location.
    Append,

    /// Write data over existing data if any.
    ///
    /// This is the behavior commonly expected by most [Write](std::io::Write) implementations,
    /// as such this is also the default for [SectionData](crate::core::data::SectionData).
    Overwrite
}

/// Opaque variant intended to manipulate section data in the form of standard IO operations.
pub trait SectionData: Read + Write + Seek {
    /// Loads this section into memory.
    ///
    /// # Errors
    ///
    /// An [Error](std::io::Error) is returned if the section could not be loaded.
    fn load_in_memory(&mut self) -> Result<Vec<u8>> {
        let mut data: Vec<u8> = Vec::new();
        self.read_to_end(&mut data)?;
        Ok(data)
    }

    //fn set_write_mode(&mut self, mode: WriteMode);

    /// Truncates this section of `size` bytes. The new section size is returned.
    ///
    /// Once the section is truncated, bytes to be read after the truncation point are ignored.
    ///
    /// # Arguments
    ///
    /// * `size`: the number of bytes to chop from the end of the section.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::io::{Read, Seek, SeekFrom, Write};
    /// use bpx::core::{AutoSectionData, SectionData};
    ///
    /// let mut data = AutoSectionData::new();
    /// data.write_all(b"test").unwrap();
    /// data.truncate(2);
    /// data.seek(SeekFrom::Start(0)).unwrap();
    /// let mut buf = [0; 4];
    /// data.read(&mut buf).unwrap();
    /// assert_eq!(std::str::from_utf8(&buf).unwrap(), "te\0\0");
    /// ```
    fn truncate(&mut self, size: usize) -> Result<usize>;

    /// Shifts all bytes after cursor, in the section, to the left or to the right.
    ///
    /// If this operation fails the data in the section may appear partially shifted.
    ///
    /// # Arguments
    ///
    /// * `shift`: the shift direction and length.
    ///
    /// returns: Result<(), Error>
    ///
    /// # Errors
    ///
    /// An [Error](std::io::Error) is returned if a read, write or seek operation has failed.
    fn shift(&mut self, shift: Shift) -> Result<()> {
        let cursor = self.stream_position()?;
        let mut buf = [0; SHIFT_BUF_SIZE];
        match shift {
            Shift::Left(mut length) => {
                if length > cursor as u32 {
                    length = cursor as u32;
                }
                let mut destination = cursor - length as u64;
                let mut source = cursor;
                loop {
                    self.seek(SeekFrom::Start(source))?;
                    let len = self.read_fill(&mut buf)?;
                    if len == 0 {
                        break;
                    }
                    source += len as u64;
                    self.seek(SeekFrom::Start(destination))?;
                    self.write_all(&buf[..len])?;
                    destination += len as u64;
                }
            },
            Shift::Right(length) => {
                let size = self.size() as u64;
                let mut source = size;
                let mut destination = source + length as u64;
                while source > cursor {
                    let nextsize = std::cmp::min(SHIFT_BUF_SIZE as u64, source - cursor);
                    self.seek(SeekFrom::Start(source - nextsize))?;
                    self.read_exact(&mut buf[..nextsize as usize])?;
                    self.seek(SeekFrom::Start(destination - nextsize))?;
                    self.write_all(&buf[..nextsize as usize])?;
                    source -= nextsize;
                    destination -= nextsize;
                }
            }
        }
        Ok(())
    }

    /// Returns the current size of this section.
    fn size(&self) -> usize;
}

pub use auto::AutoSectionData;
use crate::utils::ReadFill;

#[cfg(test)]
mod tests {
    use std::io::{Read, Seek, SeekFrom, Write};
    use crate::core::{AutoSectionData, SectionData};
    use crate::core::data::Shift;

    const SEED: &str = "This is a test.";

    #[test]
    fn basic_shift_left() {
        let mut data = AutoSectionData::new();
        data.write_all(SEED.as_bytes()).unwrap();
        data.seek(SeekFrom::End(-4)).unwrap();
        data.shift(Shift::Left(2)).unwrap();
        data.seek(SeekFrom::Start(0)).unwrap();
        let mut buf = [0; SEED.len()];
        data.read(&mut buf).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "This is aest.t.");
    }

    #[test]
    fn basic_shift_right() {
        let mut data = AutoSectionData::new();
        data.write_all(SEED.as_bytes()).unwrap();
        data.seek(SeekFrom::End(-4)).unwrap();
        data.shift(Shift::Right(2)).unwrap();
        data.seek(SeekFrom::Start(0)).unwrap();
        let mut buf = [0; SEED.len() + 2];
        data.read(&mut buf).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "This is a tesest.");
    }

    #[test]
    fn long_shift_left() {
        let mut data = AutoSectionData::new();
        data.write_all(SEED.as_bytes()).unwrap();
        data.seek(SeekFrom::Start(4)).unwrap();
        data.shift(Shift::Left(2)).unwrap();
        data.seek(SeekFrom::Start(0)).unwrap();
        let mut buf = [0; SEED.len()];
        data.read(&mut buf).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "Th is a test.t.");
    }

    #[test]
    fn long_shift_right() {
        let mut data = AutoSectionData::new();
        data.write_all(SEED.as_bytes()).unwrap();
        data.seek(SeekFrom::Start(4)).unwrap();
        data.shift(Shift::Right(2)).unwrap();
        data.seek(SeekFrom::Start(0)).unwrap();
        let mut buf = [0; SEED.len() + 2];
        data.read(&mut buf).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "This i is a test.");
    }

    #[test]
    fn zero_shift_left() {
        let mut data = AutoSectionData::new();
        data.write_all(SEED.as_bytes()).unwrap();
        data.seek(SeekFrom::Start(0)).unwrap();
        data.shift(Shift::Left(2)).unwrap();
        data.seek(SeekFrom::Start(0)).unwrap();
        let mut buf = [0; SEED.len()];
        data.read(&mut buf).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "This is a test.");
    }

    #[test]
    fn zero_shift_right() {
        let mut data = AutoSectionData::new();
        data.write_all(SEED.as_bytes()).unwrap();
        data.seek(SeekFrom::Start(0)).unwrap();
        data.shift(Shift::Right(2)).unwrap();
        data.seek(SeekFrom::Start(0)).unwrap();
        let mut buf = [0; SEED.len() + 2];
        data.read(&mut buf).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "ThThis is a test.");
    }
}
