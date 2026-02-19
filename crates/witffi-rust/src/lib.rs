//! # witffi-rust
//!
//! Generates Rust `extern "C"` scaffolding from WIT interface definitions.
//!
//! This crate produces:
//! - A Rust trait that library authors implement
//! - `#[repr(C)]` struct/enum types for all WIT records/variants
//! - `#[no_mangle] pub unsafe extern "C" fn` wrappers that marshal
//!   between C ABI types and Rust types
//! - Corresponding `free_*` functions for heap-allocated returns
//! - A C header file

pub mod generate;

pub use generate::RustGenerator;

/// The contents of `witffi_types.h`, embedded at compile time.
///
/// Consumers (CLI, build scripts) can write this to disk alongside the
/// generated `ffi.h` so that downstream C / Swift / Kotlin code finds
/// both headers in a single directory.
pub const WITFFI_TYPES_HEADER: &str = include_str!("../../witffi-types/witffi_types.h");
