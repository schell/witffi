//! Build task library for eip681 FFI binding generation.
//!
//! Provides a single [`generate`] function that produces all FFI artifacts
//! (Rust scaffolding, C headers, Kotlin bindings, Swift bindings) from the
//! eip681 WIT definition. Configuration values and relative output paths are
//! hardcoded to the eip681 example layout.
//!
//! Used by both the `xtask` binary (`cargo xtask generate`) and
//! `examples/eip681-ffi/build.rs`.

use std::path::Path;

use snafu::prelude::*;

/// Errors that can occur during binding generation.
#[derive(Debug, Snafu)]
pub enum Error {
    /// Failed to load the WIT file.
    #[snafu(display("failed to load WIT from {path}"))]
    LoadWit {
        path: String,
        source: witffi_core::Error,
    },

    /// Failed to generate Rust scaffolding.
    #[snafu(display("failed to generate Rust scaffolding"))]
    GenerateRust {
        source: witffi_rust::generate::Error,
    },

    /// Failed to generate the C header.
    #[snafu(display("failed to generate C header"))]
    GenerateCHeader {
        source: witffi_rust::generate::Error,
    },

    /// Failed to generate Kotlin bindings.
    #[snafu(display("failed to generate Kotlin bindings"))]
    GenerateKotlin {
        source: witffi_kotlin::generate::Error,
    },

    /// Failed to generate Swift bindings.
    #[snafu(display("failed to generate Swift bindings"))]
    GenerateSwift {
        source: witffi_swift::generate::Error,
    },

    /// Failed to generate the Swift module map.
    #[snafu(display("failed to generate Swift module map"))]
    GenerateModuleMap {
        source: witffi_swift::generate::Error,
    },

    /// An I/O operation failed.
    #[snafu(display("i/o error: {path}"))]
    Io {
        path: String,
        source: std::io::Error,
    },
}

// ---- Hardcoded eip681 configuration ----

/// C function name prefix.
const C_PREFIX: &str = "zcash_eip681";

/// C type name prefix.
const C_TYPE_PREFIX: &str = "Ffi";

/// Kotlin/JNI package name.
const KOTLIN_PACKAGE: &str = "zcash.eip681";

/// Library name for JNI `System.loadLibrary()`.
const LIBRARY_NAME: &str = "eip681_ffi";

// ---- Relative paths from workspace root ----

/// Path to the WIT definition file.
const WIT_PATH: &str = "wit/eip681.wit";

/// Rust scaffolding output.
const RUST_OUTPUT: &str = "examples/eip681-ffi/src/ffi.rs";

/// C header output (FFI crate root).
const FFI_HEADER: &str = "examples/eip681-ffi/ffi.h";

/// witffi_types.h output (FFI crate root).
const FFI_TYPES_HEADER: &str = "examples/eip681-ffi/witffi_types.h";

/// Kotlin bindings output.
const KOTLIN_OUTPUT: &str = "examples/eip681-kotlin/src/Bindings.kt";

/// Swift bindings output.
const SWIFT_OUTPUT: &str = "examples/eip681-swift/Sources/Eip681/Bindings.swift";

/// Swift C header output.
const SWIFT_HEADER: &str = "examples/eip681-swift/Sources/CZcashEip681/include/ffi.h";

/// Swift witffi_types.h output.
const SWIFT_TYPES_HEADER: &str =
    "examples/eip681-swift/Sources/CZcashEip681/include/witffi_types.h";

/// Swift module map output.
const SWIFT_MODULE_MAP: &str =
    "examples/eip681-swift/Sources/CZcashEip681/include/module.modulemap";

// ---- Public API ----

/// Generate all eip681 FFI artifacts from the WIT definition.
///
/// Produces Rust scaffolding, C headers, Kotlin bindings, and Swift bindings,
/// writing each to its expected location relative to `workspace_root`.
///
/// # Errors
///
/// Returns an error if WIT loading, code generation, or file I/O fails.
pub fn generate(workspace_root: &Path) -> Result<(), Error> {
    let wit_path = workspace_root.join(WIT_PATH);
    let (resolve, world_id) = witffi_core::load_wit(&wit_path).context(LoadWitSnafu {
        path: wit_path.display().to_string(),
    })?;

    // ---- Rust scaffolding ----

    let rust_config = witffi_rust::generate::RustConfig {
        c_prefix: C_PREFIX.to_string(),
        c_type_prefix: C_TYPE_PREFIX.to_string(),
        kotlin_package: Some(KOTLIN_PACKAGE.to_string()),
        library_name: Some(LIBRARY_NAME.to_string()),
    };
    let rust_generator = witffi_rust::RustGenerator::new(&resolve, world_id, rust_config);

    let rust_code = rust_generator.generate().context(GenerateRustSnafu)?;
    let rust_path = workspace_root.join(RUST_OUTPUT);
    write_file(&rust_path, &rust_code)?;
    eprintln!("Wrote {}", rust_path.display());

    // Format the generated file so `cargo fmt --check` stays clean.
    let _ = std::process::Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .arg(&rust_path)
        .status();

    // ---- C headers (FFI crate) ----

    let c_header = rust_generator
        .generate_c_header()
        .context(GenerateCHeaderSnafu)?;

    let ffi_header_path = workspace_root.join(FFI_HEADER);
    write_file(&ffi_header_path, &c_header)?;
    eprintln!("Wrote {}", ffi_header_path.display());

    let ffi_types_path = workspace_root.join(FFI_TYPES_HEADER);
    write_file(&ffi_types_path, witffi_rust::WITFFI_TYPES_HEADER)?;
    eprintln!("Wrote {}", ffi_types_path.display());

    // ---- Kotlin bindings ----

    let kotlin_config = witffi_kotlin::generate::KotlinConfig {
        kotlin_package: Some(KOTLIN_PACKAGE.to_string()),
        lib_name: LIBRARY_NAME.to_string(),
    };
    let kotlin_generator = witffi_kotlin::KotlinGenerator::new(&resolve, world_id, kotlin_config);
    let kotlin_code = kotlin_generator.generate().context(GenerateKotlinSnafu)?;

    let kotlin_path = workspace_root.join(KOTLIN_OUTPUT);
    ensure_parent_dir(&kotlin_path)?;
    write_file(&kotlin_path, &kotlin_code)?;
    eprintln!("Wrote {}", kotlin_path.display());

    // ---- Swift bindings ----

    let swift_config = witffi_swift::generate::SwiftConfig {
        c_prefix: C_PREFIX.to_string(),
        c_type_prefix: C_TYPE_PREFIX.to_string(),
    };
    let swift_generator = witffi_swift::SwiftGenerator::new(&resolve, world_id, swift_config);

    let swift_code = swift_generator.generate().context(GenerateSwiftSnafu)?;
    let swift_path = workspace_root.join(SWIFT_OUTPUT);
    ensure_parent_dir(&swift_path)?;
    write_file(&swift_path, &swift_code)?;
    eprintln!("Wrote {}", swift_path.display());

    // ---- Swift C headers ----

    let swift_header_path = workspace_root.join(SWIFT_HEADER);
    ensure_parent_dir(&swift_header_path)?;
    write_file(&swift_header_path, &c_header)?;
    eprintln!("Wrote {}", swift_header_path.display());

    let swift_types_path = workspace_root.join(SWIFT_TYPES_HEADER);
    write_file(&swift_types_path, witffi_rust::WITFFI_TYPES_HEADER)?;
    eprintln!("Wrote {}", swift_types_path.display());

    // ---- Swift module map ----

    let module_map = swift_generator
        .generate_module_map()
        .context(GenerateModuleMapSnafu)?;
    let module_map_path = workspace_root.join(SWIFT_MODULE_MAP);
    write_file(&module_map_path, &module_map)?;
    eprintln!("Wrote {}", module_map_path.display());

    Ok(())
}

// ---- Helpers ----

/// Write content to a file, wrapping I/O errors with the path.
fn write_file(path: &Path, content: &str) -> Result<(), Error> {
    std::fs::write(path, content).context(IoSnafu {
        path: path.display().to_string(),
    })
}

/// Ensure the parent directory of `path` exists, creating it if necessary.
fn ensure_parent_dir(path: &Path) -> Result<(), Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context(IoSnafu {
            path: parent.display().to_string(),
        })?;
    }
    Ok(())
}
