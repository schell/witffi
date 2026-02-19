//! Build script that generates FFI scaffolding from the eip681 WIT definition.
//!
//! Uses the `witffi-core` and `witffi-rust` library APIs to parse the WIT file
//! and generate Rust `repr(C)` types, a trait, and a registration macro.

use std::path::Path;

fn main() {
    let wit_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../wit/eip681.wit");

    println!("cargo::rerun-if-changed={}", wit_path.display());

    let (resolve, world_id) = witffi_core::load_wit(&wit_path).expect("failed to load eip681.wit");

    let config = witffi_rust::generate::RustConfig {
        c_prefix: "zcash_eip681".to_string(),
        c_type_prefix: "Ffi".to_string(),
    };

    let generator = witffi_rust::RustGenerator::new(&resolve, world_id, config);

    // Generate Rust scaffolding
    let rust_code = generator.generate().expect("failed to generate Rust code");
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    let rust_path = Path::new(&out_dir).join("ffi.rs");
    std::fs::write(&rust_path, &rust_code).expect("failed to write ffi.rs");

    // Generate C header
    let c_header = generator
        .generate_c_header()
        .expect("failed to generate C header");
    let header_path = Path::new(&out_dir).join("ffi.h");
    std::fs::write(&header_path, &c_header).expect("failed to write ffi.h");

    // Also copy the header to the crate root for easy consumption
    let crate_header = Path::new(env!("CARGO_MANIFEST_DIR")).join("ffi.h");
    std::fs::write(&crate_header, &c_header).expect("failed to write ffi.h to crate root");
}
