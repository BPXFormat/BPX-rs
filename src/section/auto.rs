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
    cell::{Cell, RefCell, RefMut},
    ops::{Deref, DerefMut}
};

use crate::{
    section::{new_section_data, Error, Section, SectionData},
    Handle
};

pub struct Ref<'a>
{
    r: RefMut<'a, Box<dyn SectionData>>,
    size: &'a Cell<usize>,
    modified: &'a Cell<bool>
}

impl<'a> Ref<'a>
{
    pub fn as_mut(&mut self) -> &mut dyn SectionData
    {
        return self.r.deref_mut().deref_mut();
    }

    pub fn as_ref(&self) -> &dyn SectionData
    {
        return self.r.deref().deref();
    }
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
        self.modified.set(true);
    }
}

/// The default implementation of the Section trait.
pub struct AutoSection
{
    data: RefCell<Box<dyn SectionData>>,
    size: Cell<usize>,
    modified: Cell<bool>,
    handle: Handle
}

impl AutoSection
{
    /// Creates a new AutoSection.
    ///
    /// **NOTE: Not intended to be called in user code.**
    ///
    /// # Arguments
    ///
    /// * `size`: the size of the section.
    /// * `handle`: the section handle.
    ///
    /// returns: Result<AutoSection, Error>
    ///
    /// # Errors
    ///
    /// Returns [Error](std::io::Error) in case some IO error has occurred when the section
    /// is created as a temporary file.
    pub fn new(size: u32, handle: Handle) -> Result<AutoSection, std::io::Error>
    {
        let data = new_section_data(Some(size))?;
        return Ok(AutoSection {
            data: RefCell::new(data),
            size: Cell::new(0),
            modified: Cell::new(false),
            handle
        });
    }

    /// Opens this section and returns a reference to it.
    pub fn open(&self) -> Result<Ref<'_>, Error>
    {
        let size = self.size.get();
        if size > super::data::MEMORY_THRESHOLD as usize {
            let mut old = self.realloc(size as u32)?;
            let mut data = self.open()?;
            std::io::copy(&mut old, &mut *data)?;
        }
        if let Ok(r) = self.data.try_borrow_mut() {
            return Ok(Ref {
                r,
                size: &self.size,
                modified: &self.modified
            });
        }
        return Err(Error::AlreadyOpen);
    }

    /// Returns true if this section has been modified then resets the modified flag to false.
    ///
    /// **NOTE: Not intended to be called in user code.**
    pub fn modified(&self) -> bool
    {
        return self.modified.replace(false);
    }
}

impl Section for AutoSection
{
    fn size(&self) -> usize
    {
        return self.size.get();
    }

    fn realloc(&self, size: u32) -> Result<Box<dyn SectionData>, Error>
    {
        let data;
        if size == 0 {
            data = new_section_data(None);
        } else {
            data = new_section_data(Some(size));
        }
        return match data {
            Err(e) => Err(Error::Io(e)),
            Ok(v) => {
                {
                    if self.open().is_err() {
                        return Err(Error::AlreadyOpen);
                    }
                }
                let old = self.data.replace(v);
                self.size.set(0);
                Ok(old)
            }
        };
    }

    fn handle(&self) -> Handle
    {
        return self.handle;
    }
}
