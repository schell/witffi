//! # witffi-kotlin
//!
//! Generates Kotlin/Android bindings from WIT interface definitions.
//!
//! This crate produces a single `Bindings.kt` file containing:
//! - Kotlin data classes for WIT records
//! - Sealed class hierarchies for WIT variants
//! - Enum classes for WIT enums
//! - Inline value classes for WIT flags
//! - A backend interface and implementation class with JNI `external fun`
//!   declarations
//!
//! The generated Kotlin code assumes a corresponding Rust JNI bridge
//! (produced by `witffi-rust`'s `witffi_register_jni!` macro) is linked
//! via `System.loadLibrary()`.

pub mod generate;

pub use generate::KotlinGenerator;
