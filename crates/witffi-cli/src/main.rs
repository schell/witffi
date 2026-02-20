//! witffi CLI — generate native FFI bindings from WIT definitions.

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use snafu::prelude::*;

type Result<T, E = snafu::Whatever> = std::result::Result<T, E>;

#[derive(Parser)]
#[command(
    name = "witffi",
    about = "Generate native FFI bindings from WIT definitions"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate bindings for a target language.
    Generate {
        /// Path to a WIT file or directory.
        #[arg(long, short)]
        wit: PathBuf,

        /// Target language to generate bindings for.
        #[arg(long, short)]
        lang: Language,

        /// Output directory for generated files.
        #[arg(long, short)]
        output: PathBuf,

        /// Prefix for C function names (e.g. "zcash_eip681").
        #[arg(long, default_value = "witffi")]
        c_prefix: String,

        /// Prefix for C type names (e.g. "Ffi").
        #[arg(long, default_value = "Ffi")]
        c_type_prefix: String,

        /// Kotlin package name (e.g. "zcash.eip681").
        ///
        /// If not specified, derived from the WIT package name.
        /// Used by both `--lang rust` (for JNI class paths in the macro)
        /// and `--lang kotlin` (for the `package` declaration).
        #[arg(long)]
        kotlin_package: Option<String>,

        /// Library name for `System.loadLibrary()` / JNI loading.
        ///
        /// Used by both `--lang rust` (embedded in JNI macro) and
        /// `--lang kotlin` (in the `Bindings.kt` init block).
        #[arg(long)]
        lib_name: Option<String>,
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum Language {
    /// Generate Rust scaffolding (idiomatic types, trait, dual macros) + C header.
    Rust,
    /// Generate Swift bindings.
    Swift,
    /// Generate Kotlin/Android bindings (Bindings.kt only).
    Kotlin,
}

#[snafu::report]
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            wit,
            lang,
            output,
            c_prefix,
            c_type_prefix,
            kotlin_package,
            lib_name,
        } => {
            let (resolve, world_id) = witffi_core::load_wit(&wit)
                .with_whatever_context(|_| format!("loading WIT from {}", wit.display()))?;

            std::fs::create_dir_all(&output).with_whatever_context(|_| {
                format!("creating output directory {}", output.display())
            })?;

            match lang {
                Language::Rust => {
                    let rust_config = witffi_rust::generate::RustConfig {
                        c_prefix: c_prefix.clone(),
                        c_type_prefix: c_type_prefix.clone(),
                        kotlin_package,
                        library_name: lib_name,
                    };
                    let rust_generator =
                        witffi_rust::RustGenerator::new(&resolve, world_id, rust_config);

                    let rust_code = rust_generator
                        .generate()
                        .whatever_context("generating Rust code")?;
                    let rust_path = output.join("ffi.rs");
                    std::fs::write(&rust_path, &rust_code)
                        .with_whatever_context(|_| format!("writing {}", rust_path.display()))?;
                    eprintln!("Wrote {}", rust_path.display());

                    let c_header = rust_generator
                        .generate_c_header()
                        .whatever_context("generating C header")?;
                    let header_path = output.join("ffi.h");
                    std::fs::write(&header_path, &c_header)
                        .with_whatever_context(|_| format!("writing {}", header_path.display()))?;
                    eprintln!("Wrote {}", header_path.display());

                    let types_path = output.join("witffi_types.h");
                    std::fs::write(&types_path, witffi_rust::WITFFI_TYPES_HEADER)
                        .with_whatever_context(|_| format!("writing {}", types_path.display()))?;
                    eprintln!("Wrote {}", types_path.display());
                }

                Language::Swift => {
                    // Swift needs C headers as well as Swift bindings.
                    let rust_config = witffi_rust::generate::RustConfig {
                        c_prefix: c_prefix.clone(),
                        c_type_prefix: c_type_prefix.clone(),
                        kotlin_package: None,
                        library_name: None,
                    };
                    let rust_generator =
                        witffi_rust::RustGenerator::new(&resolve, world_id, rust_config);

                    let c_header = rust_generator
                        .generate_c_header()
                        .whatever_context("generating C header")?;
                    let header_path = output.join("ffi.h");
                    std::fs::write(&header_path, &c_header)
                        .with_whatever_context(|_| format!("writing {}", header_path.display()))?;
                    eprintln!("Wrote {}", header_path.display());

                    let types_path = output.join("witffi_types.h");
                    std::fs::write(&types_path, witffi_rust::WITFFI_TYPES_HEADER)
                        .with_whatever_context(|_| format!("writing {}", types_path.display()))?;
                    eprintln!("Wrote {}", types_path.display());

                    let swift_config = witffi_swift::generate::SwiftConfig {
                        c_prefix,
                        c_type_prefix,
                    };
                    let swift_generator =
                        witffi_swift::SwiftGenerator::new(&resolve, world_id, swift_config);

                    let swift_code = swift_generator
                        .generate()
                        .whatever_context("generating Swift code")?;
                    let swift_path = output.join("Bindings.swift");
                    std::fs::write(&swift_path, &swift_code)
                        .with_whatever_context(|_| format!("writing {}", swift_path.display()))?;
                    eprintln!("Wrote {}", swift_path.display());

                    let module_map = swift_generator
                        .generate_module_map()
                        .whatever_context("generating module map")?;
                    let map_path = output.join("module.modulemap");
                    std::fs::write(&map_path, &module_map)
                        .with_whatever_context(|_| format!("writing {}", map_path.display()))?;
                    eprintln!("Wrote {}", map_path.display());
                }

                Language::Kotlin => {
                    let kotlin_config = witffi_kotlin::generate::KotlinConfig {
                        kotlin_package,
                        lib_name: lib_name.unwrap_or_else(|| "witffi".to_string()),
                    };
                    let kotlin_generator =
                        witffi_kotlin::KotlinGenerator::new(&resolve, world_id, kotlin_config);

                    let kotlin_code = kotlin_generator
                        .generate()
                        .whatever_context("generating Kotlin code")?;
                    let kotlin_path = output.join("Bindings.kt");
                    std::fs::write(&kotlin_path, &kotlin_code)
                        .with_whatever_context(|_| format!("writing {}", kotlin_path.display()))?;
                    eprintln!("Wrote {}", kotlin_path.display());
                }
            }
        }
    }

    Ok(())
}
