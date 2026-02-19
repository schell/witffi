//! Build script that generates FFI scaffolding from the eip681 WIT definition.
//!
//! Uses the `witffi-core` and `witffi-rust` library APIs to parse the WIT file
//! and generate Rust `repr(C)` types, a trait, and a registration macro.

use std::path::Path;

fn main() {
    let cargo_manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let wit_path = cargo_manifest_dir.join("../../wit/eip681.wit");

    println!("cargo::rerun-if-changed={}", wit_path.display());

    let (resolve, world_id) = witffi_core::load_wit(&wit_path).expect("failed to load eip681.wit");

    let config = witffi_rust::generate::RustConfig {
        c_prefix: "zcash_eip681".to_string(),
        c_type_prefix: "Ffi".to_string(),
    };

    let generator = witffi_rust::RustGenerator::new(&resolve, world_id, config);

    // Generate Rust scaffolding
    let rust_code = generator.generate().expect("failed to generate Rust code");
    let src_dir = cargo_manifest_dir.join("src");
    let rust_path = Path::new(&src_dir).join("ffi.rs");
    std::fs::write(&rust_path, &rust_code).expect("failed to write ffi.rs");

    // Generate C header
    let c_header = generator
        .generate_c_header()
        .expect("failed to generate C header");
    // Copy the header to the crate root for easy consumption
    let crate_header = cargo_manifest_dir.join("ffi.h");
    std::fs::write(&crate_header, &c_header).expect("failed to write ffi.h to crate root");

    // Copy the shared types header alongside ffi.h so downstream consumers
    // can glob-copy *.h and get everything they need.
    let types_header = cargo_manifest_dir.join("witffi_types.h");
    std::fs::write(&types_header, witffi_rust::WITFFI_TYPES_HEADER)
        .expect("failed to write witffi_types.h to crate root");
}
