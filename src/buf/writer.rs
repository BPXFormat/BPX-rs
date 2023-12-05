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

use crate::core::{AutoSectionData, DEFAULT_MEMORY_THRESHOLD};
use std::io::Result;
use std::io::{Seek, SeekFrom, Write};

/// A BufWriter which supports converting a [Write] only stream into a [Write] + [Seek] for use in a
/// BPX [Container](crate::core::Container).
pub struct BufWriter<T> {
    inner: T,
    buffer: AutoSectionData,
}

impl<T> BufWriter<T> {
    /// Creates a new instance of a [BufWriter].
    ///
    /// # Arguments
    ///
    /// * `inner`: the [Write] backend to turn into both a [Write] and a [Seek].
    ///
    /// # Examples
    ///
    /// Correct use of the [BufWriter]:
    ///
    /// ```
    /// use std::io::Write;
    /// use bpx::buf::{BufReader, BufWriter};
    /// use bpx::package::Package;
    ///
    /// // Create a buffer.
    /// let buffer = BufWriter::new(Vec::new());
    /// // Write a BPXP into the buffer.
    /// let mut package = Package::create(buffer).unwrap();
    /// package.save().unwrap();
    /// // Flush the buffer so the data is written into the underlying Vec<u8>.
    /// let mut buffer = package.into_inner().into_inner();
    /// buffer.flush().unwrap();
    /// // Read the byte block back.
    /// let block = buffer.into_inner();
    /// Package::open(BufReader::new(&*block).unwrap()).unwrap();
    /// ```
    ///
    /// Don't forget to flush the [BufWriter] to the underlying backend or the package will fail to load:
    ///
    /// ```should_panic
    /// use std::io::Write;
    /// use bpx::buf::{BufReader, BufWriter};
    /// use bpx::package::Package;
    ///
    /// // Create a buffer.
    /// let buffer = BufWriter::new(Vec::new());
    /// // Write a BPXP into the buffer.
    /// let mut package = Package::create(buffer).unwrap();
    /// package.save().unwrap();
    /// // Extract the buffer without flushing it.
    /// let mut buffer = package.into_inner().into_inner();
    /// // Read the byte block back.
    /// let block = buffer.into_inner();
    /// // Of course, the last unwrap on the package will panic as the buffer wasn't let a chance
    /// // to fill the byte block.
    /// Package::open(BufReader::new(&*block).unwrap()).unwrap();
    /// ```
    pub fn new(inner: T) -> BufWriter<T> {
        BufWriter {
            inner,
            buffer: AutoSectionData::new(DEFAULT_MEMORY_THRESHOLD as _),
        }
    }

    /// Returns the inner [Write] backend.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: Write> Write for BufWriter<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.buffer.flush()?;
        self.buffer.seek(SeekFrom::Start(0))?;
        std::io::copy(&mut self.buffer, &mut self.inner).map(|_| ())
    }
}

impl<T: Write> Seek for BufWriter<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.buffer.seek(pos)
    }

    fn stream_position(&mut self) -> Result<u64> {
        self.buffer.stream_position()
    }
}
