//! # witffi-swift
//!
//! Generates Swift bindings from WIT interface definitions.
//!
//! This crate produces:
//! - Swift structs/enums matching WIT records/variants
//! - Swift function wrappers that call the C FFI layer
//! - Automatic memory management via defer/cleanup patterns

pub mod generate;

pub use generate::SwiftGenerator;
