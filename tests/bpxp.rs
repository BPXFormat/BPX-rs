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

use std::io::{Cursor, Seek, SeekFrom};
use bpx::package::{OpenOptions, Package};
use bpx::package::util::unpack_string;

#[test]
#[cfg(feature = "package")]
fn test_bpxp_string_corruption_1() {
    //Create a copy of the file in RAM.
    let file = std::fs::read("./tests/test.bpx").unwrap();
    let buffer = Cursor::new(file);

    //Load the newly copied buffer and perform the first request: create a new object called "BadName".
    let mut package = Package::open(OpenOptions::new(buffer).revert_on_save_failure(true)).unwrap();
    package.load_metadata().unwrap();
    {
        let objects = package.objects().unwrap();
        let object = objects.find("BadName").unwrap();
        assert!(object.is_none());
    }
    package.objects().unwrap();
    package.objects_mut().unwrap().create("BadName", b"This is a test".as_ref()).unwrap();
    package.load_and_save().unwrap();

    let mut buffer = package.into_inner().into_inner();
    buffer.seek(SeekFrom::Start(0)).unwrap();

    //Re-load the buffer and perform the second request: create a new object named "TestObject1".
    let mut package = Package::open(OpenOptions::new(buffer).revert_on_save_failure(true)).unwrap();
    package.load_metadata().unwrap();
    {
        let objects = package.objects().unwrap();
        let object = objects.find("TestObject1").unwrap();
        assert!(object.is_none());
    }
    package.objects().unwrap();
    package.objects_mut().unwrap().create("TestObject1", b"This is a new test".as_ref()).unwrap();
    package.load_and_save().unwrap();

    let mut buffer = package.into_inner().into_inner();
    buffer.seek(SeekFrom::Start(0)).unwrap();

    //Re-load the buffer and perform the third and last request: remove the object named "BadName".
    let mut package = Package::open(OpenOptions::new(buffer).revert_on_save_failure(true)).unwrap();
    let objects = package.objects().unwrap();
    let obj = *objects.find("BadName").unwrap().unwrap();
    package.objects_mut().unwrap().remove(&obj);
    package.load_and_save().unwrap();

    let mut buffer = package.into_inner().into_inner();
    buffer.seek(SeekFrom::Start(0)).unwrap();

    //Final integrity check
    let package = Package::open(buffer).unwrap();
    let objects = package.objects().unwrap();
    assert_eq!(objects.len(), 3);
    assert_eq!(objects.load_name(&objects[0]).unwrap(), "test_new");
    assert_eq!(objects.load_name(&objects[1]).unwrap(), "bpx");
    assert_eq!(objects.load_name(&objects[2]).unwrap(), "TestObject1");
    let s = unpack_string(&objects, &objects[2]).unwrap();
    assert_eq!(s, "This is a new test");
}

#[test]
#[cfg(feature = "package")]
fn test_bpxp_string_corruption_2() {
    //Create a copy of the file in RAM.
    let file = std::fs::read("./tests/test.bpx").unwrap();
    let buffer = Cursor::new(file);

    //Load the newly copied buffer and perform the first request: create a new object called "BadName".
    let mut package = Package::open(OpenOptions::new(buffer).revert_on_save_failure(true)).unwrap();
    package.load_metadata().unwrap();
    {
        let objects = package.objects().unwrap();
        let object = objects.find("BadName").unwrap();
        assert!(object.is_none());
    }
    package.objects().unwrap();
    package.objects_mut().unwrap().create("BadName", b"This is a test".as_ref()).unwrap();
    package.load_and_save().unwrap();

    let mut buffer = package.into_inner().into_inner();
    buffer.seek(SeekFrom::Start(0)).unwrap();

    //Re-load the buffer and perform the second request: remove the object named "BadName".
    let mut package = Package::open(OpenOptions::new(buffer).revert_on_save_failure(true)).unwrap();
    let objects = package.objects().unwrap();
    let obj = *objects.find("BadName").unwrap().unwrap();
    package.objects_mut().unwrap().remove(&obj);
    package.load_and_save().unwrap();

    let mut buffer = package.into_inner().into_inner();
    buffer.seek(SeekFrom::Start(0)).unwrap();

    //Re-load the buffer and perform the third and last request: create a new object named "TestObject1".
    let mut package = Package::open(OpenOptions::new(buffer).revert_on_save_failure(true)).unwrap();
    package.load_metadata().unwrap();
    {
        let objects = package.objects().unwrap();
        let object = objects.find("TestObject1").unwrap();
        assert!(object.is_none());
    }
    package.objects().unwrap();
    package.objects_mut().unwrap().create("TestObject1", b"This is a new test".as_ref()).unwrap();
    package.load_and_save().unwrap();

    let mut buffer = package.into_inner().into_inner();
    buffer.seek(SeekFrom::Start(0)).unwrap();

    //Final integrity check
    let package = Package::open(buffer).unwrap();
    let objects = package.objects().unwrap();
    assert_eq!(objects.len(), 3);
    assert_eq!(objects.load_name(&objects[0]).unwrap(), "test_new");
    assert_eq!(objects.load_name(&objects[1]).unwrap(), "bpx");
    assert_eq!(objects.load_name(&objects[2]).unwrap(), "TestObject1");
    let s = unpack_string(&objects, &objects[2]).unwrap();
    assert_eq!(s, "This is a new test");
}

#[test]
#[cfg(feature = "package")]
fn test_bpxp_string_corruption_3() {
    //Create a copy of the file in RAM.
    let file = std::fs::read("./tests/test.bpx").unwrap();
    let buffer = Cursor::new(file);

    //Load the newly copied buffer and perform the first request: create a new object called "bod".
    let mut package = Package::open(OpenOptions::new(buffer).revert_on_save_failure(true)).unwrap();
    package.load_metadata().unwrap();
    {
        let objects = package.objects().unwrap();
        let object = objects.find("bod").unwrap();
        assert!(object.is_none());
    }
    package.objects().unwrap();
    package.objects_mut().unwrap().create("bod", b"This is a test".as_ref()).unwrap();
    package.load_and_save().unwrap();

    let mut buffer = package.into_inner().into_inner();
    buffer.seek(SeekFrom::Start(0)).unwrap();

    //Re-load the buffer and perform the second request: remove the object named "bod".
    let mut package = Package::open(OpenOptions::new(buffer).revert_on_save_failure(true)).unwrap();
    let objects = package.objects().unwrap();
    let (i, _) = objects.iter().enumerate().find(|(_, obj)| objects.load_name(obj).unwrap() == "bod").unwrap();
    package.objects_mut().unwrap().remove_at(i);
    package.load_and_save().unwrap();

    let mut buffer = package.into_inner().into_inner();
    buffer.seek(SeekFrom::Start(0)).unwrap();

    //Re-load the buffer and perform the third and last request: create a new object named "bad".
    let mut package = Package::open(OpenOptions::new(buffer).revert_on_save_failure(true)).unwrap();
    package.load_metadata().unwrap();
    {
        let objects = package.objects().unwrap();
        let object = objects.find("bad").unwrap();
        assert!(object.is_none());
    }
    package.objects().unwrap();
    package.objects_mut().unwrap().create("bad", b"This is a new test".as_ref()).unwrap();
    package.load_and_save().unwrap();

    let mut buffer = package.into_inner().into_inner();
    buffer.seek(SeekFrom::Start(0)).unwrap();

    //Final integrity check
    let package = Package::open(buffer).unwrap();
    let objects = package.objects().unwrap();
    assert_eq!(objects.len(), 3);
    assert_eq!(objects.load_name(&objects[0]).unwrap(), "test_new");
    assert_eq!(objects.load_name(&objects[1]).unwrap(), "bpx");
    assert_eq!(objects.load_name(&objects[2]).unwrap(), "bad");
    let s = unpack_string(&objects, &objects[2]).unwrap();
    assert_eq!(s, "This is a new test");
}
