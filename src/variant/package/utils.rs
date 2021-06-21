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

use std::{
    fs::{metadata, read_dir, File},
    path::Path
};

use crate::{
    strings::{get_name_from_dir_entry, get_name_from_path},
    variant::package::PackageEncoder,
    Result
};

/// Packs a file or folder in this BPXP with the given virtual name
///
/// *this functions prints some information to standard output as a way to debug data compression issues*
///
/// # Arguments
///
/// * `encoder` - the BPX [Encoder](crate::encoder::Encoder) backend to use
/// * `source` - the source [Path](std::path::Path) to pack
/// * `vname` - the virtual name for the root source path
///
/// # Returns
///
/// * nothing if the operation succeeded
/// * an [Error](crate::error::Error) in case of system error
pub fn pack_file_vname<TBackend: crate::encoder::IoBackend>(
    package: &mut PackageEncoder<TBackend>,
    vname: &str,
    source: &Path
) -> Result<()>
{
    let md = metadata(source)?;
    if md.is_file() {
        #[cfg(feature = "debug-log")]
        println!("Writing file {} with {} byte(s)", vname, md.len());
        let mut fle = File::open(source)?;
        package.pack_object(&vname, &mut fle)?;
    } else {
        let entries = read_dir(source)?;
        for rentry in entries {
            let entry = rentry?;
            let mut s = String::from(vname);
            s.push('/');
            s.push_str(&get_name_from_dir_entry(&entry));
            pack_file_vname(package, &s, &entry.path())?;
        }
    }
    return Ok(());
}

/// Packs a file or folder in this BPXP, automatically computing the virtual name from the source path file name
///
/// *this functions prints some information to standard output as a way to debug data compression issues*
///
/// # Arguments
///
/// * `encoder` - the BPX [Encoder](crate::encoder::Encoder) backend to use
/// * `source` - the source [Path](std::path::Path) to pack
///
/// # Returns
///
/// * nothing if the operation succeeded
/// * an [Error](crate::error::Error) in case of system error
pub fn pack_file<TBackend: crate::encoder::IoBackend>(
    package: &mut PackageEncoder<TBackend>,
    source: &Path
) -> Result<()>
{
    return pack_file_vname(package, &get_name_from_path(source)?, source);
}
