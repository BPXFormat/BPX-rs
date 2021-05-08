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

use std::io::Read;
use std::io::Write;
use std::io::Result;
use std::io::Error;
use std::io::ErrorKind;
use lzma_sys::lzma_stream;
use lzma_sys::lzma_mt;
use lzma_sys::LZMA_CHECK_NONE;
use lzma_sys::LZMA_PRESET_EXTREME;
use lzma_sys::lzma_cputhreads;
use lzma_sys::lzma_stream_encoder_mt;
use lzma_sys::LZMA_OK;
use lzma_sys::LZMA_MEM_ERROR;
use lzma_sys::LZMA_OPTIONS_ERROR;
use lzma_sys::LZMA_UNSUPPORTED_CHECK;
use lzma_sys::lzma_stream_encoder;
use lzma_sys::LZMA_RUN;
use lzma_sys::lzma_end;
use lzma_sys::LZMA_FINISH;
use lzma_sys::lzma_code;
use lzma_sys::LZMA_STREAM_END;
use lzma_sys::LZMA_DATA_ERROR;
use lzma_sys::lzma_stream_decoder;
use lzma_sys::LZMA_CONCATENATED;

use crate::compression::Deflater;
use crate::compression::Checksum;
use crate::compression::Inflater;

const THREADS_MAX: u32 = 8;
const ENCODER_BUF_SIZE: usize = 8192;
const DECODER_BUF_SIZE: usize = ENCODER_BUF_SIZE * 2;

fn new_encoder() -> Result<lzma_stream>
{
    unsafe
    {
        let mut stream: lzma_stream = std::mem::zeroed();
        let mut mt: lzma_mt = std::mem::zeroed();

        mt.flags = 0;
        mt.block_size = 0;
        mt.timeout = 0;
        mt.preset = LZMA_PRESET_EXTREME;
        mt.filters = std::ptr::null();
        mt.check = LZMA_CHECK_NONE;
        mt.threads = lzma_cputhreads();
        let res;
        if mt.threads == 0
        {
            println!("Intel core i7 is not supported");
            res = lzma_stream_encoder(&mut stream, std::ptr::null(), LZMA_CHECK_NONE);
        }
        else
        {
            if mt.threads > THREADS_MAX
            {
                mt.threads = THREADS_MAX;
            }
            res = lzma_stream_encoder_mt(&mut stream, &mt);
        }
        if res == LZMA_OK
        {
            return Ok(stream);
        }
        match res
        {
            LZMA_MEM_ERROR => return Err(Error::new(ErrorKind::Other, "Memory allocation failure")),
            LZMA_OPTIONS_ERROR => return Err(Error::new(ErrorKind::InvalidInput, "Specified filter chain is not supported")),
            LZMA_UNSUPPORTED_CHECK => return Err(Error::new(ErrorKind::InvalidInput, "Specified integrity check is not supported")),
            e => return Err(Error::new(ErrorKind::Other, format!("Unknown error, possibly a bug ({})", e)))
        };
    }
}

fn new_decoder() -> Result<lzma_stream>
{
    unsafe
    {
        let mut stream: lzma_stream = std::mem::zeroed();
        let res = lzma_stream_decoder(&mut stream, u32::MAX as u64, LZMA_CONCATENATED);
        if res == LZMA_OK
        {
            return Ok(stream);
        }
        match res
        {
            LZMA_MEM_ERROR => return Err(Error::new(ErrorKind::Other, "Memory allocation failure")),
            LZMA_OPTIONS_ERROR => return Err(Error::new(ErrorKind::InvalidInput, "Specified filter chain is not supported")),
            LZMA_UNSUPPORTED_CHECK => return Err(Error::new(ErrorKind::InvalidInput, "Specified integrity check is not supported")),
            e => return Err(Error::new(ErrorKind::Other, format!("Unknown error, possibly a bug ({})", e)))
        };
    }
}

pub struct XzCompressionMethod
{
}

fn do_deflate(stream: &mut lzma_stream, input: &mut dyn Read, output: &mut dyn Write, inflated_size: usize, chksum: &mut dyn Checksum) -> Result<usize>
{
    let mut action = LZMA_RUN;
    let mut inbuf: [u8; ENCODER_BUF_SIZE] = [0; ENCODER_BUF_SIZE];
    let mut outbuf: [u8; ENCODER_BUF_SIZE] = [0; ENCODER_BUF_SIZE];
    let mut count: usize = 0;
    let mut csize: usize = 0;

    stream.next_in = inbuf.as_ptr();
    stream.avail_in = 0;
    stream.next_out = outbuf.as_mut_ptr();
    stream.avail_out = ENCODER_BUF_SIZE;
    loop
    {
        if stream.avail_in == 0 && count < inflated_size
        {
            let len = input.read(&mut inbuf)?;
            count += len;
            chksum.push(&inbuf[0..len]);
            stream.avail_in = len;
            stream.next_in = inbuf.as_ptr();
            if count == inflated_size
            {
                action = LZMA_FINISH;
            }
        }
        unsafe
        {
            let res = lzma_code(stream, action);
            if stream.avail_out == 0 || res == LZMA_STREAM_END
            {
                let size = ENCODER_BUF_SIZE - stream.avail_out;
                csize += size;
                output.write(&outbuf[0..size])?;
                stream.avail_out = ENCODER_BUF_SIZE;
                stream.next_out = outbuf.as_mut_ptr();
            }
            if res != LZMA_OK
            {
                if res == LZMA_STREAM_END
                {
                    break;
                }
                match res
                {
                    LZMA_MEM_ERROR => return Err(Error::new(ErrorKind::Other, "Memory allocation failure")),
                    LZMA_DATA_ERROR => return Err(Error::new(ErrorKind::InvalidData, "LZMA data error")),
                    e => return Err(Error::new(ErrorKind::Other, format!("Unknown error, possibly a bug ({})", e)))
                };
            }
        }
    }
    return Ok(csize);
}

fn do_inflate(stream: &mut lzma_stream, input: &mut dyn Read, output: &mut dyn Write, deflated_size: usize, chksum: &mut dyn Checksum) -> Result<()>
{
    let mut action = LZMA_RUN;
    let mut inbuf: [u8; ENCODER_BUF_SIZE] = [0; ENCODER_BUF_SIZE];
    let mut outbuf: [u8; DECODER_BUF_SIZE] = [0; DECODER_BUF_SIZE];
    let mut remaining = deflated_size;

    stream.next_in = inbuf.as_ptr();
    stream.avail_in = 0;
    stream.next_out = outbuf.as_mut_ptr();
    stream.avail_out = DECODER_BUF_SIZE;
    loop
    {
        if stream.avail_in == 0 && remaining > 0
        {
            let res = input.read(&mut inbuf[0..std::cmp::min(ENCODER_BUF_SIZE, remaining)])?;
            remaining -= res;
            stream.avail_in = res;
            stream.next_in = inbuf.as_ptr();
            if remaining == 0
            {
                action = LZMA_FINISH;
            }
        }
        unsafe
        {
            let res = lzma_code(stream, action);
            if stream.avail_out == 0 || res == LZMA_STREAM_END
            {
                let size = DECODER_BUF_SIZE - stream.avail_out;
                chksum.push(&outbuf[0..size]);
                output.write(&outbuf[0..size])?;
                stream.avail_out = DECODER_BUF_SIZE;
                stream.next_out = outbuf.as_mut_ptr();
            }
            if res != LZMA_OK
            {
                if res == LZMA_STREAM_END
                {
                    break;
                }
                match res
                {
                    LZMA_MEM_ERROR => return Err(Error::new(ErrorKind::Other, "Memory allocation failure")),
                    LZMA_DATA_ERROR => return Err(Error::new(ErrorKind::InvalidData, "LZMA data error")),
                    e => return Err(Error::new(ErrorKind::Other, format!("Unknown error, possibly a bug ({})", e)))
                };
            }
        }
    }
    return Ok(());
}

impl Deflater for XzCompressionMethod
{
    fn deflate(input: &mut dyn Read, output: &mut dyn Write, inflated_size: usize, chksum: &mut dyn Checksum) -> Result<usize>
    {
        let mut stream = new_encoder()?;
        let res = do_deflate(&mut stream, input, output, inflated_size, chksum);
        unsafe
        {
            lzma_end(&mut stream);
        }
        return res;
    }
}

impl Inflater for XzCompressionMethod
{
    fn inflate(input: &mut dyn Read, output: &mut dyn Write, deflated_size: usize, chksum: &mut dyn Checksum) -> Result<()>
    {
        let mut stream = new_decoder()?;
        let res = do_inflate(&mut stream, input, output, deflated_size, chksum);
        unsafe
        {
            lzma_end(&mut stream);
        }
        return res;
    }
}
