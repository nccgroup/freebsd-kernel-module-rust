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

use core::alloc::{GlobalAlloc, Layout};

pub struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        kernel_sys::malloc(
            layout.size(),
            &mut kernel_sys::M_DEVBUF[0],
            kernel_sys::M_WAITOK,
        ) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        kernel_sys::free(
            ptr as *mut libc::c_void,
            &mut kernel_sys::M_DEVBUF[0],
        );
    }
}

/// from `sys/malloc.h`
/// ```c,ignore
/// #define    M_NOWAIT    0x0001        /* do not block */
/// #define    M_WAITOK    0x0002        /* ok to block */
/// #define    M_ZERO        0x0100        /* bzero the allocation */
/// #define    M_NOVM        0x0200        /* don't ask VM for pages */
/// #define    M_USE_RESERVE    0x0400        /* can alloc out of reserve memory */
/// #define    M_NODUMP    0x0800        /* don't dump pages in this allocation */
/// #define    M_FIRSTFIT    0x1000        /* only for vmem, fast fit */
/// #define    M_BESTFIT    0x2000        /* only for vmem, low fragmentation */
/// #define    M_EXEC        0x4000        /* allocate executable space */
/// #define    M_NEXTFIT    0x8000        /* only for vmem, follow cursor */
/// ```

#[alloc_error_handler]
fn oom(_layout: Layout) -> ! {
    panic!("Out of memory!");
}
