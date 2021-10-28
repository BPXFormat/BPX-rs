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

use std::cell::{Cell, RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use crate::section::{Error, new_section_data, Section, SectionData};
use crate::SectionHandle;

pub struct Ref<'a>
{
    r: RefMut<'a, Box<dyn SectionData>>,
    size: &'a Cell<usize>
}

impl<'a> Deref for Ref<'a>
{
    type Target = dyn SectionData;

    fn deref(&self) -> &Self::Target
    {
        return self.r.deref().deref();
    }
}

impl<'a> DerefMut for Ref<'a>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        return self.r.deref_mut().deref_mut();
    }
}

impl<'a> Drop for Ref<'a>
{
    fn drop(&mut self)
    {
        let s = self.r.deref().size();
        self.size.set(s);
    }
}

pub struct AutoSection
{
    data: RefCell<Box<dyn SectionData>>,
    size: Cell<usize>,
    handle: SectionHandle
}

impl AutoSection
{
    pub fn new(size: u32, handle: SectionHandle) -> Result<AutoSection, std::io::Error>
    {
        let data;
        if size == 0 {
            data = new_section_data(None)?;
        } else {
            data = new_section_data(Some(size))?;
        }
        return Ok(AutoSection {
            data: RefCell::new(data),
            size: Cell::new(0),
            handle
        });
    }

    pub fn open(&self) -> Result<Ref<'_>, Error>
    {
        if let Ok(r) = self.data.try_borrow_mut() {
            return Ok(Ref {
                r,
                size: &self.size
            });
        }
        return Err(Error::AlreadyOpen);
    }
}

impl Section for AutoSection
{
    fn size(&self) -> usize
    {
        return self.size.get();
    }

    fn realloc(&self, size: u32) -> Result<(), Error>
    {
        let data;
        if size == 0 {
            data = new_section_data(None);
        } else {
            data = new_section_data(Some(size));
        }
        match data {
            Err(e) => return Err(Error::Io(e)),
            Ok(v) => {
                {
                    if let Err(_) = self.open() {
                        return Err(Error::AlreadyOpen);
                    }
                }
                self.data.replace(v);
                self.size.set(0);
            }
        }
        return Ok(());
    }

    fn handle(&self) -> SectionHandle
    {
        return self.handle;
    }
}
