//! Build script that generates FFI scaffolding and Kotlin bindings from the
//! eip681 WIT definition.
//!
//! Uses the `witffi-core`, `witffi-rust`, and `witffi-kotlin` library APIs to
//! parse the WIT file and generate:
//! - `src/ffi.rs` — Rust types, trait, and dual registration macros
//! - `ffi.h` + `witffi_types.h` — C headers for Swift/Go consumers
//! - `../eip681-kotlin/src/Bindings.kt` — Kotlin model classes and JNI bridge

use std::path::Path;

fn main() {
    let cargo_manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let wit_path = cargo_manifest_dir.join("../../wit/eip681.wit");

    println!("cargo::rerun-if-changed={}", wit_path.display());

    let (resolve, world_id) = witffi_core::load_wit(&wit_path).expect("failed to load eip681.wit");

    // ---- Rust scaffolding ----

    let rust_config = witffi_rust::generate::RustConfig {
        c_prefix: "zcash_eip681".to_string(),
        c_type_prefix: "Ffi".to_string(),
        kotlin_package: Some("zcash.eip681".to_string()),
        library_name: Some("eip681_ffi".to_string()),
    };

    let rust_generator = witffi_rust::RustGenerator::new(&resolve, world_id, rust_config);

    let rust_code = rust_generator
        .generate()
        .expect("failed to generate Rust code");
    let src_dir = cargo_manifest_dir.join("src");
    let rust_path = Path::new(&src_dir).join("ffi.rs");
    std::fs::write(&rust_path, &rust_code).expect("failed to write ffi.rs");

    // Format the generated file so `cargo fmt --check` stays clean.
    let _ = std::process::Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .arg(&rust_path)
        .status();

    // ---- C headers ----

    let c_header = rust_generator
        .generate_c_header()
        .expect("failed to generate C header");
    let crate_header = cargo_manifest_dir.join("ffi.h");
    std::fs::write(&crate_header, &c_header).expect("failed to write ffi.h to crate root");

    let types_header = cargo_manifest_dir.join("witffi_types.h");
    std::fs::write(&types_header, witffi_rust::WITFFI_TYPES_HEADER)
        .expect("failed to write witffi_types.h to crate root");

    // ---- Kotlin bindings ----

    let kotlin_config = witffi_kotlin::generate::KotlinConfig {
        kotlin_package: Some("zcash.eip681".to_string()),
        lib_name: "eip681_ffi".to_string(),
    };

    let kotlin_generator = witffi_kotlin::KotlinGenerator::new(&resolve, world_id, kotlin_config);
    let kotlin_code = kotlin_generator
        .generate()
        .expect("failed to generate Kotlin code");

    let kotlin_dir = cargo_manifest_dir.join("../eip681-kotlin/src");
    std::fs::create_dir_all(&kotlin_dir).expect("failed to create eip681-kotlin/src directory");
    std::fs::write(kotlin_dir.join("Bindings.kt"), &kotlin_code)
        .expect("failed to write Bindings.kt");
}
