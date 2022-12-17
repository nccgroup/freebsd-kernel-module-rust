// Copyright (c) 2022 NCC Group
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this
//    list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//
// Based on public domain code by Johannes Lundberg

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use bsd_kernel::character_device::{CDev, CharacterDevice};
use bsd_kernel::debugln;
use bsd_kernel::io::{Read, Write};
use bsd_kernel::module::{ModuleEvents, SharedModule};
use bsd_kernel::uio::{UioReader, UioWriter};
use lazy_static::lazy_static;

lazy_static! {
    // Object created on first access (which is module load callback)
    pub static ref MODULE:
        SharedModule<Hello> = SharedModule::new();
}

#[derive(Debug)]
pub struct Hello {
    data: String,
    _cdev: CDev<Hello>,
}

impl ModuleEvents for Hello {
    fn load() -> Option<Self> {
        debugln!("[module.rs] Hello::load");

        // MODULE has been fully initialised here
        // so we can clone it safely
        let m = MODULE.clone();

        if let Some(cdev) = CDev::new_with_delegate("rustmodule", m) {
            Some(Hello {
                data: "Default hello message\n".to_string(),
                _cdev: cdev,
            })
        } else {
            debugln!(
                "[module.rs] Hello::load: Failed to create character device"
            );
            None
        }
    }

    fn unload(&mut self) {
        debugln!("[module.rs] Hello::unload");
    }
}

impl CharacterDevice for Hello {
    fn open(&mut self) {
        // debugln!("[module.rs] Hello::open");
    }
    fn close(&mut self) {
        // debugln!("[module.rs] Hello::close");
    }
    fn read(&mut self, uio: &mut UioWriter) {
        // debugln!("[module.rs] Hello::read");
        match uio.write_all(&self.data.as_bytes()) {
            Ok(()) => (),
            Err(e) => debugln!("{}", e),
        }
    }
    fn write(&mut self, uio: &mut UioReader) {
        // debugln!("[module.rs] Hello::write");
        self.data.clear();
        match uio.read_to_string(&mut self.data) {
            Ok(x) => {
                debugln!(
                    "Read {} bytes. Setting new message to `{}`",
                    x,
                    inner.data
                )
            }
            Err(e) => debugln!("{:?}", e),
        }
    }
}
