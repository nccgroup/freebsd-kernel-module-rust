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

use crate::cstr_ref;
use crate::module::SharedModule;
use crate::uio::{UioReader, UioWriter};
use alloc::boxed::Box;
use core::prelude::v1::*;
use core::{fmt, mem, ptr};
use libc::c_int;

/// ```c,ignore
/// /*
///  * Character device switch table
///  */
/// struct cdevsw {
///     int            d_version;
///     u_int            d_flags;
///     const char        *d_name;
///     d_open_t        *d_open;
///     d_fdopen_t        *d_fdopen;
///     d_close_t        *d_close;
///     d_read_t        *d_read;
///     d_write_t        *d_write;
///     d_ioctl_t        *d_ioctl;
///     d_poll_t        *d_poll;
///     d_mmap_t        *d_mmap;
///     d_strategy_t        *d_strategy;
///     dumper_t        *d_dump;
///     d_kqfilter_t        *d_kqfilter;
///     d_purge_t        *d_purge;
///     d_mmap_single_t        *d_mmap_single;
///
///     int32_t            d_spare0[3];
///     void            *d_spare1[3];
///
///     /* These fields should not be messed with by drivers */
///     LIST_HEAD(, cdev)    d_devs;
///     int            d_spare2;
///     union {
///         struct cdevsw        *gianttrick;
///         SLIST_ENTRY(cdevsw)    postfree_list;
///     } __d_giant;
/// };
/// ```

pub trait CharacterDevice {
    fn open(&mut self);
    fn close(&mut self);
    fn read(&mut self, uio: &mut UioWriter);
    fn write(&mut self, uio: &mut UioReader);
}

pub struct CDev<T>
where
    T: CharacterDevice,
{
    cdev: ptr::NonNull<kernel_sys::cdev>,
    delegate: SharedModule<T>,
}

impl<T> CDev<T>
where
    T: CharacterDevice,
{
    pub fn new_with_delegate(
        name: &'static str,
        delegate: SharedModule<T>,
    ) -> Option<Box<Self>> {
        let cdevsw_raw: *mut kernel_sys::cdevsw = {
            let mut c: kernel_sys::cdevsw = unsafe { mem::zeroed() };
            c.d_open = Some(cdev_open::<T>);
            c.d_close = Some(cdev_close::<T>);
            c.d_read = Some(cdev_read::<T>);
            c.d_write = Some(cdev_write::<T>);
            c.d_version = kernel_sys::D_VERSION as i32;
            c.d_name = "helloworld".as_ptr() as *mut i8;
            Box::into_raw(Box::new(c))
        };

        let cdev_raw: *mut kernel_sys::cdev = unsafe {
            kernel_sys::make_dev(
                cdevsw_raw,
                0,
                kernel_sys::UID_ROOT as u32,
                kernel_sys::GID_WHEEL as u32,
                0o660,
                cstr_ref!(name).as_ptr() as *mut i8,
            )
        };

        match cdev_raw.is_null() {
            true => {
                // Convert cdevsw back to Box so memory can be freed
                let _cdevsw = unsafe { Box::from_raw(cdevsw_raw) };
                None
            }
            false => {
                let cdev = Box::new(CDev {
                    cdev: ptr::NonNull::new(cdev_raw).unwrap(),
                    delegate,
                });
                unsafe {
                    (*cdev_raw).si_drv1 =
                        &*cdev as *const CDev<T> as *mut libc::c_void
                };
                Some(cdev)
            }
        }
    }
}

impl<T> fmt::Debug for CDev<T>
where
    T: CharacterDevice,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CDev {{ cdev: {:?}, ... }}", self.cdev.as_ptr())
    }
}

impl<T> Drop for CDev<T>
where
    T: CharacterDevice,
{
    fn drop(&mut self) {
        // debugln!("[kernel.rs] CDev::drop");

        // Assign only to clarify what type we're dealing with...
        let dev: *mut kernel_sys::cdev = self.cdev.as_ptr();

        // Back to Box so cdevsw memory is freed
        let _cdevsw: Box<kernel_sys::cdevsw> =
            unsafe { Box::from_raw((*dev).si_devsw) };

        // debugln!("[kernel.rs] CDev::drop calling destroy_dev. ptr={:?}", dev.as_ptr());
        unsafe { kernel_sys::destroy_dev(dev) };
    }
}

// File operations callbacks
extern "C" fn cdev_open<T>(
    dev: *mut kernel_sys::cdev,
    _oflags: c_int,
    _devtype: c_int,
    _td: *mut kernel_sys::thread,
) -> c_int
where
    T: CharacterDevice,
{
    // debugln!("cdev_open");
    let cdev: &CDev<T> = unsafe { &*((*dev).si_drv1 as *const CDev<T>) };
    if let Some(mut m) = cdev.delegate.lock() {
        m.open();
    }
    0
}

#[allow(unused)]
extern "C" fn cdev_fdopen(
    _dev: *mut kernel_sys::cdev,
    _oflags: c_int,
    _td: *mut kernel_sys::thread,
    _fp: *mut kernel_sys::file,
) -> c_int {
    // debugln!("cdev_fdopen");
    0
}

extern "C" fn cdev_close<T>(
    dev: *mut kernel_sys::cdev,
    _fflag: c_int,
    _devtype: c_int,
    _td: *mut kernel_sys::thread,
) -> c_int
where
    T: CharacterDevice,
{
    // debugln!("cdev_close");
    let cdev: &CDev<T> = unsafe { &*((*dev).si_drv1 as *const CDev<T>) };
    if let Some(mut m) = cdev.delegate.lock() {
        m.close();
    }
    0
}

extern "C" fn cdev_read<T>(
    dev: *mut kernel_sys::cdev,
    uio: *mut kernel_sys::uio,
    _ioflag: c_int,
) -> c_int
where
    T: CharacterDevice,
{
    // debugln!("cdev_read");
    let cdev: &CDev<T> = unsafe { &*((*dev).si_drv1 as *const CDev<T>) };
    if let Some(mut m) = cdev.delegate.lock() {
        m.read(&mut UioWriter::new(uio));
    }
    0
}

extern "C" fn cdev_write<T>(
    dev: *mut kernel_sys::cdev,
    uio: *mut kernel_sys::uio,
    _ioflag: c_int,
) -> c_int
where
    T: CharacterDevice,
{
    // debugln!("cdev_write");
    let cdev: &CDev<T> = unsafe { &*((*dev).si_drv1 as *const CDev<T>) };
    if let Some(mut m) = cdev.delegate.lock() {
        m.write(unsafe { &mut UioReader::new(uio) });
    }
    0
}
