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

//! Traits and interfaces for modules

use crate::error::Error;
use alloc::sync::Arc;
use core::convert::{TryFrom, TryInto};
use core::ops::{Deref, DerefMut};
use core::prelude::v1::*;
use core::{fmt, ptr};
use kernel_sys::{
    modeventtype_MOD_LOAD, modeventtype_MOD_QUIESCE, modeventtype_MOD_SHUTDOWN,
    modeventtype_MOD_UNLOAD,
};
use spin::{Mutex, MutexGuard};

/// The module event types
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ModuleEventType {
    /// Module is being loaded
    Load = modeventtype_MOD_LOAD,
    /// Module is being unloaded
    Unload = modeventtype_MOD_UNLOAD,
    /// The system is shutting down
    Shutdown = modeventtype_MOD_SHUTDOWN,
    /// The module is about to be unloaded - returning an error from the
    /// QUIESCE event causes kldunload to cancel the unload (unless forced
    /// with -f)
    Quiesce = modeventtype_MOD_QUIESCE,
}

impl TryFrom<i32> for ModuleEventType {
    type Error = Error;
    fn try_from(input: i32) -> Result<Self, Self::Error> {
        use ModuleEventType::*;
        #[allow(non_upper_case_globals)]
        match input.try_into()? {
            modeventtype_MOD_LOAD => Ok(Load),
            modeventtype_MOD_UNLOAD => Ok(Unload),
            modeventtype_MOD_SHUTDOWN => Ok(Shutdown),
            modeventtype_MOD_QUIESCE => Ok(Quiesce),
            _ => Err(Error::ConversionError("Invalid value for modeventtype")),
        }
    }
}

impl ModuleEventType {
    /// Attempt to convert an i32 representation of a module event into
    /// a `ModuleEventType` enum variant. Returns `None` if the value is
    /// not a valid module event
    pub fn from_i32(n: i32) -> Option<ModuleEventType> {
        ModuleEventType::try_from(n).ok()
    }
}

/// Functions to handle each type of module event
///
/// TODO: functions for SHUTDOWN and QUIESCE with default implementations
pub trait ModuleEvents {
    /// Function called when the module is loaded
    fn load(&mut self);
    /// Function called when the module is unloaded
    fn unload(&mut self);
}

pub struct LockedModule<'a, T: Sized + 'a> {
    guard: MutexGuard<'a, Option<T>>,
}

//impl<'a, T: Sized> LockedModule<'a, T> {}

impl<'a, T: Sized> Deref for LockedModule<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.guard.as_ref().unwrap()
    }
}

impl<'a, T: Sized> DerefMut for LockedModule<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.guard.as_mut().unwrap()
    }
}

impl<'a, T> Drop for LockedModule<'a, T> {
    fn drop(&mut self) {
        // debugln!("[kernel.rs] LockedModule::drop");
    }
}

impl<'a, T> core::fmt::Debug for LockedModule<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LockedModule {{ guard: MutexGuard<Option<T>> }}")
    }
}

/// Wrapper to protect a module behind a mutex
#[derive(Clone, Debug, Default)]
pub struct SharedModule<T> {
    inner: Arc<Mutex<Option<T>>>,
}

impl<T> SharedModule<T> {
    pub fn new(data: T) -> Self {
        SharedModule {
            inner: Arc::new(Mutex::new(Some(data))),
        }
    }

    pub fn inner(&self) -> Arc<Mutex<Option<T>>> {
        self.inner.clone()
    }

    pub fn lock(&self) -> Option<LockedModule<T>> {
        self.inner
            .lock()
            .ok()
            .map(|guard| LockedModule { guard })
    }

    pub fn cleanup(&self) {
        {
            let _ = self.inner.lock().take();
        }
        // Safe to do this in kldunload callback?
        // If we don't, we'll leak 64 byte Mutex struct (maybe not a disaster...)
        unsafe {
            let ptr: *mut Arc<Mutex<Option<T>>> = &self.inner
                as *const Arc<Mutex<Option<T>>>
                as *mut Arc<Mutex<Option<T>>>;
            ptr::drop_in_place(ptr);
        }
    }
}

impl<T> Drop for SharedModule<T> {
    fn drop(&mut self) {
        // debugln!("[kernel.rs] SharedModule::drop");
    }
}

unsafe impl<T> Sync for SharedModule<T> {}
unsafe impl<T> Send for SharedModule<T> {}
