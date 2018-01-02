//! Library for versioning symbols in native libraries.
//!
//! This library provides a means by which native (C, C++, assembly) symbols can
//! be automatically version mangled. This allows multiple versions of a Rust
//! library with C, C++, and assembly code to be used in one application.
//!
//! # How it Works
//!
//! The general idea is that all symbols in C, C++, and assembly code will be
//! mangled with the current crate version. For instance, a symbol `some_func`
//! will become `some_func_v1_23_2_beta`. On the Rust side, all symbols will be
//! linked to using the mangled name. The C, C++, and assembly name mangling
//! will happen transparently. On the Rust side, all `extern` block will be
//! modified to use this crate's `versioned_extern!` macro.
//!
//! # Detailed Usage
//!
//! This library works best with the [`cc`] crate, and we assume you're using
//! `cc` to build external source files in this documentation.
//!
//! ## Dependencies
//!
//! First, you'll need to add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! native_versioning = "*"
//!
//! [build_dependencies]
//! native_versioning = { version = "*", features = ["build"] }
//! ```
//!
//! ## `build.rs`
//!
//! Your `build.rs` will generate the `generated_versioned.h` file using the
//! [`write_versioned_header()`] function and build the native sources with the
//! appropriate flags.
//!
//! ### Generated Header File
//!
//! To use the [`write_versioned_header()`] function, you first need to decide
//! three things:
//!
//!   1. The name of the generated header file.
//!   2. The name of the generated macro in the header file.
//!   3. Where to store the generated header file.
//!
//! We recommend you record the first two decisions in `const`s and create a
//! function for the third:
//!
//! ```rust
//! use std::path::PathBuf;
//!
//! const GENERATED_VERSIONED_HEADER: &str = "generated_versioned.h";
//! const GENERATED_VERSIONED_MACRO: &str = "VERSIONED";
//!
//! fn generated_include_dir() -> PathBuf {
//!     const GENERATED_INCLUDE_DIR: &str = "generated_headers";
//!     PathBuf::from(env::var("OUT_DIR").unwrap()).join(GENERATED_INCLUDE_DIR)
//! }
//! ```
//!
//! Finally, in `main`, generate the header file:
//!
//! ```rust
//! let generated_include_dir = generated_include_dir();
//! write_versioned_header(&generated_include_dir,
//!                        GENERATED_VERSIONED_HEADER,
//!                        GENERATED_VERSIONED_MACRO)
//!     .expect("generated versioned header file");
//! ```
//!
//! The file is written to `generated_include_dir()/GENERATED_VERSION_HEADER`.
//!
//! ### Versioning Symbols
//!
//! You'll now need to use the generated header to construct a header that
//! versions all symbols in your C, C++, and assembly source files. This file
//! should look as follows:
//!
//! ```C
//! #include <generated_versioned.h>
//!
//! #define foo VERSIONED(foo)
//! #define bar VERSIONED(bar)
//!
//! // only necessary if using native MacOS symbols in assembly files
//! #define _foo VERSIONED(_foo)
//! #define _bar VERSIONED(_bar)
//! ```
//!
//! Note that the filename in the `#include` corresponds to the
//! `GENERATED_VERSIONED_HEADER` `const` and the `VERSIONED` macro being called
//! corresponds to the `GENERATED_VERSIONED_MACRO` `const`.
//!
//! You might find it useful to create a function that returns a path to this
//! file:
//!
//! ```rust
//! fn custom_versioned_symbols_header() -> PathBuf {
//!      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ext").join("versioned.h")
//! }
//! ```
//!
//! ### Building Sources
//!
//! Finally, we'll tie everything together.
//!
//! In your `cc::Build` object, add the directory including the generated
//! versioned header file to the include path and use the [`include_header`]
//! method provided by this crate to include your custom versioned symbols
//! header file:
//!
//! ```rust,ignore
//! use native_versioning::HeaderInclude;
//!
//! cc::build::new()
//!     ...
//!     .include(&generated_include_dir())
//!     .include_header(&custom_versioned_symbols_header())
//!     ...
//! ```
//!
//! ### Overview
//!
//! In all, a simple `build.rs` using this crate will look as follows:
//!
//! ```rust
//! extern crate cc;
//! extern crate native_versioning;
//!
//! use std::path::{Path, PathBuf};
//!
//! use native_versioning::{HeaderInclude, write_versioned_header};
//!
//! const GENERATED_VERSIONED_HEADER: &str = "generated_versioned.h";
//! const GENERATED_VERSIONED_MACRO: &str = "VERSIONED";
//!
//! fn generated_include_dir() -> PathBuf {
//!     const GENERATED_INCLUDE_DIR: &str = "generated_headers";
//!     PathBuf::from(::std::env::var("OUT_DIR").unwrap()).join(GENERATED_INCLUDE_DIR)
//! }
//!
//! fn custom_versioned_symbols_header() -> PathBuf {
//!     PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ext").join("versioned.h")
//! }
//!
//! fn main() {
//!     let generated_include_dir = generated_include_dir();
//!     write_versioned_header(&generated_include_dir,
//!                            GENERATED_VERSIONED_HEADER,
//!                            GENERATED_VERSIONED_MACRO)
//!         .expect("generated versioned header file");
//!
//!     cc::Build::new()
//!         .file(Path::new("ext").join("foo.c"))
//!         .file(Path::new("ext").join("bar.S"))
//!         .include(&generated_include_dir)
//!         .include_header(&custom_versioned_symbols_header())
//!         .compile("foo");
//! }
//! ```
//!
//! ## Importing Mangled Symbols
//!
//! To import the versioned symbols on the Rust side, use the
//! [`versioned_extern!`] macro provided by this crate:
//!
//! ```rust
//! #[macro_use] extern crate native_versioning;
//!
//! versioned_extern! {
//!     fn foo(u8) -> u8;
//!     fn bar(*mut i16, *mut i32);
//! }
//!
//! fn main() {
//!     unsafe {
//!         println!("Number: {}", foo(10));
//!     }
//! }
//! ```
//!
//! The macro is a drop-in replacement for Rust's `extern` blocks. As a result,
//! you can take an existing codebase and simply replace all appearances of
//! `extern {` with `versioned_extern! {`.
//!
//! [`write_versioned_header()`]: fn.write_versioned_header.html
//! [`versioned_extern!`]: macro.versioned_extern.html
//!
mod versioned_extern;

#[cfg(feature = "build")]
mod build;

#[cfg(feature = "build")]
pub use build::*;
