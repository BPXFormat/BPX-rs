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

pub enum ReadError
{
    Bpx(crate::error::ReadError),
    Io(std::io::Error),
    Sd(crate::sd::ReadError),
    Strings(crate::strings::ReadError),
    InvalidArchCode(u8),
    InvalidPlatformCode(u8),
    BadVersion(u32),
    BadType(u8),
    MissingStrings,
    MissingObjectTable,

    /// Indicates the BPXP decoder reached EOS (End Of Section) while reading the object table
    Eos,

    /// Indicates an object is truncated
    Truncation,

    /// Indicates a blank string was obtained when attempting to unpack a BPXP to the file system
    BlankString
}

impl From<std::io::Error> for ReadError
{
    fn from(e: std::io::Error) -> Self
    {
        return ReadError::Io(e);
    }
}

impl From<crate::error::ReadError> for ReadError
{
    fn from(e: crate::error::ReadError) -> Self
    {
        return ReadError::Bpx(e);
    }
}

impl From<crate::strings::ReadError> for ReadError
{
    fn from(e: crate::strings::ReadError) -> Self
    {
        return ReadError::Strings(e);
    }
}

impl From<crate::sd::ReadError> for ReadError
{
    fn from(e: crate::sd::ReadError) -> Self
    {
        return ReadError::Sd(e);
    }
}

pub enum WriteError
{
    Bpx(crate::error::WriteError),
    Io(std::io::Error),
    Strings(crate::strings::WriteError),
    Sd(crate::sd::WriteError),

    //Indicates an invalid path while attempting to pack some files
    InvalidPath
}

impl From<std::io::Error> for WriteError
{
    fn from(e: std::io::Error) -> Self
    {
        return WriteError::Io(e);
    }
}

impl From<crate::error::WriteError> for WriteError
{
    fn from(e: crate::error::WriteError) -> Self
    {
        return WriteError::Bpx(e);
    }
}

impl From<crate::strings::WriteError> for WriteError
{
    fn from(e: crate::strings::WriteError) -> Self
    {
        return WriteError::Strings(e);
    }
}

impl From<crate::sd::WriteError> for WriteError
{
    fn from(e: crate::sd::WriteError) -> Self
    {
        return WriteError::Sd(e);
    }
}
