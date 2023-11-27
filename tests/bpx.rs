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

use std::{fs::File, path::Path};

use bpx::core::{header::BPX_CURRENT_VERSION, Container};

#[test]
fn attempt_write_empty_bpxp() {
    {
        let file = File::create(Path::new("./the_very_first_bpx.bpx")).unwrap();
        let mut container = Container::create(file);
        container.save().unwrap();
    }
    {
        let file = File::open(Path::new("./the_very_first_bpx.bpx")).unwrap();
        let container = Container::open(file).unwrap();
        assert_eq!(container.main_header().section_num, 0);
        assert_eq!(container.main_header().version, BPX_CURRENT_VERSION);
        assert_eq!(container.main_header().file_size, 40);
    }
}

#[test]
#[cfg(feature = "sd")]
fn sd_api_test() {
    use std::convert::TryInto;

    use bpx::sd::Value;

    let v = Value::from(None as Option<i32>);
    let v1 = Value::from("test");
    let v2 = Value::from(Some(0));
    let vu: Option<i32> = v.try_into().unwrap();
    let v1u: String = v1.try_into().unwrap();
    let v2u: Option<i32> = v2.try_into().unwrap();

    assert_eq!(vu, None);
    assert_eq!(v1u, String::from("test"));
    assert_eq!(v2u, Some(0));
}
