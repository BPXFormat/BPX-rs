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

//! Contains various utilities to be used by other modules.

use std::{io::Cursor, num::Wrapping};

/// Hash text using the hash function defined in the BPX specification for strings.
///
/// # Arguments
///
/// * `s`: the string to compute the hash of.
///
/// returns: u64
///
/// # Examples
///
/// ```
/// use bpx::utils::hash;
///
/// let s = "MyString";
/// assert_eq!(hash(s), hash("MyString"));
/// assert_eq!(hash(s), hash(s));
/// assert_ne!(hash(s), hash("Wrong"));
/// ```
pub fn hash(s: &str) -> u64
{
    let mut val: Wrapping<u64> = Wrapping(5381);

    for v in s.as_bytes() {
        val = ((val << 5) + val) + Wrapping(*v as u64);
    }
    return val.0;
}

/// Extension to include get_or_insert_with but with support for Result and errors.
pub trait OptionExtension<T>
{
    /// Inserts the value returned by `f` if the Option is None then returns a mutable reference
    /// to the value.
    ///
    /// # Arguments
    ///
    /// * `f`: the function to insert with.
    ///
    /// returns: Result<&mut T, TError>
    ///
    /// # Errors
    ///
    /// Whatever error type is `TError`.
    fn get_or_insert_with_err<TError, F: FnOnce() -> Result<T, TError>>(
        &mut self,
        f: F
    ) -> Result<&mut T, TError>;
}

impl<T> OptionExtension<T> for Option<T>
{
    fn get_or_insert_with_err<TError, F: FnOnce() -> Result<T, TError>>(
        &mut self,
        f: F
    ) -> Result<&mut T, TError>
    {
        if let None = *self {
            *self = Some(f()?);
        }

        match self {
            Some(v) => Ok(v),
            // SAFETY: a `None` variant for `self` would have been replaced by a `Some`
            // variant in the code above.
            None => unsafe { std::hint::unreachable_unchecked() }
        }
    }
}

/// Creates a new in-memory byte buffer which can be used
/// to plug as IoBackend to a BPX encoder or decoder.
///
/// # Arguments
///
/// * `size`: the initial size of the buffer; if not known use 0.
///
/// returns: Cursor<Vec<u8>>
pub fn new_byte_buf(size: usize) -> Cursor<Vec<u8>>
{
    if size > 0 {
        return Cursor::new(Vec::with_capacity(size));
    }
    return Cursor::new(Vec::new());
}

/// Allows to read into a buffer as much as possible.
///
/// *Allows the use BufReader with BPX*
pub trait ReadFill
{

    /// Reads into `buf` as much as possible.
    ///
    /// *Returns the number of bytes that could be read.*
    ///
    /// # Arguments
    ///
    /// * `buf`: the buffer to read into.
    ///
    /// returns: Result<usize, Error>
    ///
    /// # Errors
    ///
    /// Returns an [Error](std::io::Error) when read has failed.
    fn read_fill(&mut self, buf: &mut [u8]) -> std::io::Result<usize>;
}

impl<T: std::io::Read + ?Sized> ReadFill for T
{
    fn read_fill(&mut self, buf: &mut [u8]) -> std::io::Result<usize>
    {
        let mut bytes = 0;
        let mut len = self.read(buf)?;
        bytes += len;
        while len > 0 && buf.len() - len > 0 {
            len = self.read(&mut buf[len..])?;
            bytes += len;
        }
        return Ok(bytes);
    }
}
