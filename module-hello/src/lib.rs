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

#![no_std]
#![feature(default_alloc_error_handler)]

//! Example kernel module for FreeBSD written in Rust
//!
//! To build, run the following commands:
//! ```bash,ignore
//! cd bsd-rust
//! ./build.sh
//! sudo make load
//! echo "hi rust" > /dev/rustmodule
//! cat /dev/rustmodule
//! sudo make unload
//! ```

use bsd_kernel::allocator::KernelAllocator;
use bsd_kernel::module::{ModuleEventType, ModuleEvents};
use bsd_kernel::{debugln, println};
use core::panic::PanicInfo;
use libc::{c_int, c_void};
use module::MODULE;

mod module;

extern crate alloc;

#[global_allocator]
static ALLOCATOR: KernelAllocator = KernelAllocator;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        println!("Panic occurred: {}", s);
    } else {
        println!("Panic occurred");
    }

    if let Some(loc) = info.location() {
        println!("Panic at line `{}` of file `{}`", loc.line(), loc.file());
    }

    loop {}
}

/// Main event handler for module events
#[no_mangle]
pub extern "C" fn module_event(
    _module: bsd_kernel::Module,
    event: c_int,
    _arg: *mut c_void,
) -> c_int {
    // debugln!("[interface.rs] Got event {}", event);

    if let Some(ev) = ModuleEventType::from_i32(event) {
        use ModuleEventType::*;
        match ev {
            Load => {
                // debugln!("[interface.rs] MOD_LOAD");

                if let Some(mut m) = MODULE.lock() {
                    m.load();
                }
            }
            Unload => {
                // debugln!("[interface.rs] MOD_UNLOAD");

                if let Some(mut m) = MODULE.lock() {
                    m.unload();
                }

                MODULE.cleanup();
            }
            Quiesce => {
                // debugln!("[interface.rs] MOD_QUIESCE");
            }
            Shutdown => {
                // debugln!("[interface.rs] MOD_SHUTDOWN");
            }
        }
    } else {
        debugln!("[interface.rs] Undefined event");
    }
    0
}
