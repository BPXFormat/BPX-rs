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

use libz_sys::{z_stream, deflateInit_, Z_DEFAULT_COMPRESSION, Z_VERSION_ERROR, Z_STREAM_ERROR, Z_MEM_ERROR, Z_OK};
use crate::Result;
use crate::error::Error;
use crate::compression::{Checksum};
use std::io::{Read};

// Needed to bypass rust new "feature" to prevent users from using std::mem::zeroed() on UB types.
// Because this z_stream struct is repr(C) rust must guarantee ABI compatibility with C.
// That is must use pointers for function pointer. If it doesn't do this anymore, then this will cause UB in low-level C code.
// Will obviously fail on platforms/architectures not using 0 to represent NULL pointers.
unsafe fn zstream_zeroed() -> z_stream
{
    let arr: [u8; std::mem::size_of::<z_stream>()] = [0; std::mem::size_of::<z_stream>()];
    return std::mem::transmute(arr);
}

fn new_encoder() -> Result<z_stream>
{
    unsafe
    {
        let mut stream: z_stream = zstream_zeroed();
        //stream.zalloc = std::ptr::null() as _;
        //stream.zfree = std::ptr::null() as _;
        //stream.opaque = std::ptr::null() as _;
        let err = deflateInit_(&mut stream as _, Z_DEFAULT_COMPRESSION, "1.1.3".as_ptr() as _, std::mem::size_of::<z_stream>() as _);
        if err == Z_OK
        {
            return Ok(stream);
        }
        match err
        {
            Z_MEM_ERROR => return Err(Error::Deflate("Memory allocation failure")),
            Z_STREAM_ERROR=> return Err(Error::Deflate("Invalid compression level")),
            Z_VERSION_ERROR => return Err(Error::Deflate("Version mismatch")),
            _ => return Err(Error::Deflate("Unknown error, possibly a bug"))
        }
    }
}

pub struct ZlibCompressionMethod
{
}
