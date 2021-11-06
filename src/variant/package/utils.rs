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

//! BPXP utility functions.

use std::{
    fs::{metadata, read_dir, File},
    path::{Path, PathBuf}
};

use crate::{
    strings::{get_name_from_dir_entry, get_name_from_path},
    variant::{
        package::{
            error::{EosContext, ReadError, WriteError},
            object::ObjectHeader,
            PackageDecoder,
            PackageEncoder
        }
    }
};

/// Packs a file or folder in a BPXP with the given virtual name.
///
/// **This function prints some information to standard output as a way
/// to debug data compression issues unless the `debug-log` feature
/// is disabled.**
///
/// # Arguments
///
/// * `package`: the BPXP [PackageEncoder](crate::variant::package::PackageEncoder) to use.
/// * `vname`: the virtual name for the root source path.
/// * `source`: the source [Path](std::path::Path) to pack.
///
/// returns: Result<(), Error>
///
/// # Errors
///
/// A [WriteError](crate::variant::package::error::WriteError) is returned if some objects could not be packed.
pub fn pack_file_vname<TBackend: crate::encoder::IoBackend>(
    package: &mut PackageEncoder<TBackend>,
    vname: &str,
    source: &Path
) -> Result<(), WriteError>
{
    let md = metadata(source)?;
    if md.is_file() {
        #[cfg(feature = "debug-log")]
        println!("Writing file {} with {} byte(s)", vname, md.len());
        let mut fle = File::open(source)?;
        package.pack_object(vname, &mut fle)?;
    } else {
        let entries = read_dir(source)?;
        for rentry in entries {
            let entry = rentry?;
            let mut s = String::from(vname);
            s.push('/');
            s.push_str(&get_name_from_dir_entry(&entry)?);
            pack_file_vname(package, &s, &entry.path())?;
        }
    }
    Ok(())
}

/// Packs a file or folder in a BPXP, automatically computing
/// the virtual name from the source path file name.
///
/// **This function prints some information to standard output as a way
/// to debug data compression issues unless the `debug-log` feature
/// is disabled.**
///
/// # Arguments
///
/// * `package`: the BPXP [PackageEncoder](crate::variant::package::PackageEncoder) to use.
/// * `source`: the source [Path](std::path::Path) to pack.
///
/// returns: Result<(), Error>
///
/// # Errors
///
/// A [WriteError](crate::variant::package::error::WriteError) is returned if some objects could not be packed.
pub fn pack_file<TBackend: crate::encoder::IoBackend>(
    package: &mut PackageEncoder<TBackend>,
    source: &Path
) -> Result<(), WriteError>
{
    let str = get_name_from_path(source)?;
    pack_file_vname(package, str, source)
}

/// Loads an object into memory.
///
/// # Arguments
///
/// * `package`: the BPXP [PackageDecoder](crate::variant::package::PackageDecoder) to use.
/// * `obj`: the object header.
///
/// returns: Result<Vec<u8>, Error>
///
/// # Errors
///
/// A [ReadError](crate::variant::package::error::ReadError) is returned if the object could not be unpacked.
pub fn unpack_memory<TBackend: crate::decoder::IoBackend>(
    package: &mut PackageDecoder<TBackend>,
    obj: &ObjectHeader
) -> Result<Vec<u8>, ReadError>
{
    let mut v = Vec::with_capacity(obj.size as usize);
    let len = package.unpack_object(obj, &mut v)?;
    if len != obj.size {
        return Err(ReadError::Eos(EosContext::Object));
    }
    Ok(v)
}

/// Unpacks an object to the given file.
///
/// # Arguments
///
/// * `package`: the BPXP [PackageDecoder](crate::variant::package::PackageDecoder) to use.
/// * `obj`: the object header.
/// * `out`: the output [Path](std::path::Path).
///
/// returns: Result<File, Error>
///
/// # Errors
///
/// An [ReadError](crate::variant::package::error::ReadError) is returned if the object could not be unpacked.
pub fn unpack_file<TBackend: crate::decoder::IoBackend>(
    package: &mut PackageDecoder<TBackend>,
    obj: &ObjectHeader,
    out: &Path
) -> Result<File, ReadError>
{
    let mut f = File::create(out)?;
    let len = package.unpack_object(obj, &mut f)?;
    if len != obj.size {
        return Err(ReadError::Eos(EosContext::Object));
    }
    Ok(f)
}

/// Unpacks a BPXP.
///
/// **This function prints some information to standard output as a way
/// to debug a broken or incorrectly packed BPXP unless the `debug-log`
/// feature is disabled.**
///
/// # Arguments
///
/// * `package`: the BPXP [PackageDecoder](crate::variant::package::PackageDecoder) to unpack.
/// * `target`: the target [Path](std::path::Path) to extract the content to.
///
/// returns: Result<(), Error>
///
/// # Errors
///
/// An [ReadError](crate::variant::package::error::ReadError) is returned if some objects could not be unpacked.
pub fn unpack<TBackend: crate::decoder::IoBackend>(
    package: &mut PackageDecoder<TBackend>,
    target: &Path
) -> Result<(), ReadError>
{
    let (items, mut names) = package.read_object_table()?;
    for v in &items {
        let path = names.load(v)?;
        if path.is_empty() {
            return Err(ReadError::BlankString);
        }
        #[cfg(feature = "debug-log")]
        println!("Reading {} with {} byte(s)...", path, v.size);
        let mut dest = PathBuf::new();
        dest.push(target);
        dest.push(Path::new(path));
        if let Some(v) = dest.parent() {
            std::fs::create_dir_all(v)?;
        }
        unpack_file(package, v, &dest)?;
    }
    Ok(())
}
