// Copyright (c) 2023, BlockProject 3D
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

use std::cmp::min;
use std::io::{Read, Write, Result, Seek, SeekFrom, Error, ErrorKind};
use bytesutil::ReadFill;
use crate::core::{AutoSectionData, DEFAULT_MEMORY_THRESHOLD};
use crate::core::header::{MainHeader, SIZE_MAIN_HEADER, Struct};
use crate::util::UnwrapAny;

pub const MAX_IMMEDIATE_MEMORY_SIZE: u64 = 16384;

pub const MAX_MEMORY_SIZE: u64 = DEFAULT_MEMORY_THRESHOLD as _;

pub const BUF_SIZE: usize = 8192;

/// A BufReader which supports converting a [Read] only stream into a [Read] + [Seek] for use in a
/// BPX [Container](crate::core::Container).
pub struct BufReader<T> {
    inner: T,
    buffer: AutoSectionData,
    maximum_size: usize,
    cur_size: usize
}

impl<T: Read> BufReader<T> {
    /// Creates a new instance of a [BufReader].
    ///
    /// This function uses the BPX Main Header to approximate and optimize the size of the buffer
    /// required to support [Read] and [Seek] operations.
    ///
    /// # Arguments
    ///
    /// * `read`: the [Read] backend to turn into both a [Read] and a [Seek].
    ///
    /// # Examples
    ///
    /// ```
    /// use bpx::buf::BufReader;
    /// use bpx::package::Package;
    ///
    /// let broken = [0xFF as u8; 512];
    /// let reader = BufReader::new(&broken as &[u8] /* type inference in Rust is half broken */).unwrap();
    /// let res = Package::open(reader);
    /// assert!(res.is_err()); //This is expected to fail because the BPX signature s broken.
    /// ```
    pub fn new(mut read: T) -> Result<BufReader<T>> {
        if let Some(header) = MainHeader::read(&mut read).map_err(|e| e.into_value()).unwrap_any() {
            let size = header.file_size;
            if size > 0 {
                let mut buffer = AutoSectionData::new_with_size(size as _, MAX_MEMORY_SIZE as _)?;
                let mut cur_size = SIZE_MAIN_HEADER;
                buffer.write_all(&header.to_bytes())?;
                if size <= MAX_IMMEDIATE_MEMORY_SIZE {
                    let mut buf = vec![0; size as usize - SIZE_MAIN_HEADER];
                    cur_size += size as usize - SIZE_MAIN_HEADER;
                    let size = read.read_fill(&mut buf)?;
                    buffer.write_all(&buf[..size])?;
                } else {
                    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
                    let size = read.read_fill(&mut buf)?;
                    buffer.write_all(&buf[..size])?;
                    cur_size += BUF_SIZE;
                }
                buffer.seek(SeekFrom::Start(0))?;
                return Ok(BufReader {
                    inner: read,
                    buffer,
                    maximum_size: size as _,
                    cur_size
                })
            }
        }
        Ok(BufReader {
            inner: read,
            buffer: AutoSectionData::new(MAX_MEMORY_SIZE as _),
            maximum_size: 0,
            cur_size: 0
        })
    }

    fn read_into_buffer(&mut self, len: usize) -> Result<()> {
        if self.maximum_size != 0 && self.cur_size >= self.maximum_size {
            //Do not do anything if maximum size is already reached.
            return Ok(());
        }
        let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
        let size = self.inner.read_fill(&mut buf[..len])?;
        let pos = self.buffer.stream_position()?;
        self.buffer.seek(SeekFrom::End(0))?;
        self.buffer.write_all(&buf[..size])?;
        self.buffer.seek(SeekFrom::Start(pos))?;
        self.cur_size += len;
        Ok(())
    }

    /// Returns the inner [Read] backend.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Read> Read for BufReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let pos = self.buffer.stream_position()? as usize;
        while self.cur_size < pos + buf.len() {
            let len = min(BUF_SIZE, (pos + buf.len()) - self.cur_size);
            self.read_into_buffer(len)?;
        }
        self.buffer.read(buf)
    }
}

impl<T: Read> Seek for BufReader<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let pos1 = match pos {
            SeekFrom::Start(v) => v,
            SeekFrom::End(v) => {
                if self.maximum_size == 0 {
                    return Err(Error::new(ErrorKind::Unsupported, "seek from the end is unsupported on unsized streams"));
                }
                if v > 0 {
                    return Err(Error::new(ErrorKind::Unsupported, "seek past the end of the BPX is unsupported"));
                }
                self.maximum_size as u64 - ((-v) as u64)
            }
            SeekFrom::Current(v) => {
                let cur = self.buffer.stream_position()?;
                if v > 0 {
                    cur + v as u64
                } else {
                    cur - ((-v) as u64)
                }
            }
        };
        if self.maximum_size != 0 && pos1 > self.maximum_size as _ {
            return Err(Error::new(ErrorKind::Unsupported, "seek past the end of the BPX is unsupported"));
        }
        while self.cur_size < pos1 as _ {
            let len = min(BUF_SIZE, pos1 as usize - self.cur_size);
            self.read_into_buffer(len)?;
        }
        self.buffer.seek(pos)
    }

    fn stream_position(&mut self) -> Result<u64> {
        self.buffer.stream_position()
    }
}

#[cfg(test)]
mod tests {
    use crate::buf::BufReader;
    use crate::package::Package;
    use crate::util::new_byte_buf;

    #[test]
    fn basic() {
        let mut package = Package::create(new_byte_buf(1024)).unwrap();
        package.objects_mut().unwrap().create("TestObject", b"This is a test".as_ref()).unwrap();
        package.save().unwrap();
        let buffer = package.into_inner().into_inner().into_inner();
        let reader = BufReader::new(&*buffer).unwrap();
        let package = Package::open(reader).unwrap();
        let objects = package.objects().unwrap();
        assert_eq!(objects.len(), 1);
        assert_eq!(objects.load_name(objects.iter().next().unwrap()).unwrap(), "TestObject");
        let mut vec = Vec::new();
        let size = objects.load(objects.iter().next().unwrap(), &mut vec).unwrap();
        assert_eq!(&vec[..size as _], b"This is a test");
    }
}
