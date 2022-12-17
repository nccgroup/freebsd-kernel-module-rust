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
#![feature(alloc_error_handler)]

// Re-export libc and kernel_sys so that the printing macros work
pub use kernel_sys;
pub use libc;

pub use kernel_sys::module_t as Module;

extern crate alloc;

pub mod allocator;
pub mod character_device;
pub mod error;
pub mod io;
pub mod module;
pub mod uio;

/// Create a null-terminated constant string at compile time
#[macro_export]
macro_rules! cstr {
    ($arg:expr) => {
        concat!($arg, '\x00')
    };
}

/// Create a null-terminated string at runtime from any `Display` type
#[macro_export]
macro_rules! cstr_ref {
    ($arg:expr) => {
        &alloc::format!("{}\x00", $arg)
    };
}

/// Print kernel debug messages without a trailing newline
#[macro_export]
macro_rules! print {
    // Static (zero-allocation) implementation that uses compile-time `concat!()` only
    ($fmt:expr) => ({
        let msg = $crate::cstr!($fmt);
        let ptr = msg.as_ptr() as *const $crate::libc::c_char;
        unsafe {
            $crate::kernel_sys::uprintf(ptr);
        }
    });

    // Dynamic implementation that processes format arguments
    ($fmt:expr, $($arg:tt)*) => ({
        use ::core::fmt::Write;
        use $crate::io::KernelDebugWriter;
        let mut writer = KernelDebugWriter {};
        writer.write_fmt(format_args!($fmt, $($arg)*)).unwrap();
    });
}

/// Print kernel debug messages with a trailing newline
#[macro_export]
macro_rules! println {
    ($fmt:expr)              => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)+) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}

/// Print kernel debug messages without a trailing newline in debug builds
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => (if cfg!(debug_assertions) { $crate::print!($($arg)*) })
}

/// Print kernel debug messages with a trailing newline in debug builds
#[macro_export]
macro_rules! debugln {
    ($($arg:tt)*) => (if cfg!(debug_assertions) { $crate::println!($($arg)*) })
}
