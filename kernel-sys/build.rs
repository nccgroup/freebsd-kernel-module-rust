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

extern crate bindgen;

use bindgen::{Builder, MacroTypeVariation::Signed};
use std::path::PathBuf;

const FILEPATH: &str = "src/bindings.rs";

fn main() {
    let src_base = match std::env::var("SRC_BASE") {
        Ok(s) => format!("-I{s}/sys"),
        _ => "-I/usr/src/sys".to_string()
    };
    let bindings = Builder::default()
        .rustfmt_bindings(true)
        .use_core()
        .ctypes_prefix("libc")
        .size_t_is_usize(true)
        .default_macro_constant_type(Signed)
        .header("wrapper.h")
        .clang_arg("-O2")
        .clang_arg("-pipe")
        .clang_arg("-fno-strict-aliasing")
        .clang_arg("-Werror")
        .clang_arg("-D_KERNEL")
        .clang_arg("-DKLD_MODULE")
        .clang_arg("-nostdinc")
        .clang_arg("-I.")
        .clang_arg(src_base)
        .clang_arg("-I/usr/include")
        .clang_arg("-fno-common")
        .clang_arg("-fno-omit-frame-pointer")
        .clang_arg("-mno-omit-leaf-frame-pointer")
        .clang_arg("-MD")
        .clang_arg("-mcmodel=kernel")
        .clang_arg("-mno-red-zone")
        .clang_arg("-mno-mmx")
        .clang_arg("-mno-sse")
        .clang_arg("-msoft-float")
        .clang_arg("-fno-asynchronous-unwind-tables")
        .clang_arg("-ffreestanding")
        .clang_arg("-fwrapv")
        .clang_arg("-fstack-protector")
        .clang_arg("-Wall")
        .clang_arg("-Wredundant-decls")
        .clang_arg("-Wnested-externs")
        .clang_arg("-Wstrict-prototypes")
        .clang_arg("-Wmissing-prototypes")
        .clang_arg("-Wpointer-arith")
        .clang_arg("-Winline")
        .clang_arg("-Wcast-qual")
        .clang_arg("-Wundef")
        .clang_arg("-Wno-pointer-sign")
        .clang_arg("-D__printf__=__freebsd_kprintf__")
        .clang_arg("-Wmissing-include-dirs")
        .clang_arg("-fdiagnostics-show-option")
        .clang_arg("-Wno-unknown-pragmas")
        // .clang_arg("-Wno-error-tautological-compare")
        // .clang_arg("-Wno-error-empty-body")
        .clang_arg("-mno-aes")
        .clang_arg("-mno-avx")
        .clang_arg("-std=iso9899:1999")
        .generate()
        .expect("Unable to generate binding");

    let out_path = PathBuf::from(FILEPATH);
    bindings
        .write_to_file(out_path)
        .expect("Error writing bindings!");
}
