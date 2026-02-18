//! witffi CLI — generate native FFI bindings from WIT definitions.

use std::path::PathBuf;

use anyhow::Context as _;
use clap::{Parser, Subcommand, ValueEnum};

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
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum Language {
    /// Generate Rust extern "C" scaffolding + C header.
    Rust,
    /// Generate Swift bindings.
    Swift,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            wit,
            lang,
            output,
            c_prefix,
            c_type_prefix,
        } => {
            let (resolve, world_id) = witffi_core::load_wit(&wit)
                .with_context(|| format!("loading WIT from {}", wit.display()))?;

            std::fs::create_dir_all(&output)
                .with_context(|| format!("creating output directory {}", output.display()))?;

            match lang {
                Language::Rust => {
                    let config = witffi_rust::generate::RustConfig {
                        c_prefix,
                        c_type_prefix,
                    };
                    let generator = witffi_rust::RustGenerator::new(&resolve, world_id, config);

                    let rust_code = generator.generate()?;
                    let rust_path = output.join("ffi.rs");
                    std::fs::write(&rust_path, &rust_code)
                        .with_context(|| format!("writing {}", rust_path.display()))?;
                    eprintln!("Wrote {}", rust_path.display());

                    let c_header = generator.generate_c_header()?;
                    let header_path = output.join("ffi.h");
                    std::fs::write(&header_path, &c_header)
                        .with_context(|| format!("writing {}", header_path.display()))?;
                    eprintln!("Wrote {}", header_path.display());
                }
                Language::Swift => {
                    let config = witffi_swift::generate::SwiftConfig::default();
                    let generator = witffi_swift::SwiftGenerator::new(&resolve, world_id, config);
                    let swift_code = generator.generate()?;
                    let swift_path = output.join("Bindings.swift");
                    std::fs::write(&swift_path, &swift_code)
                        .with_context(|| format!("writing {}", swift_path.display()))?;
                    eprintln!("Wrote {}", swift_path.display());
                }
            }
        }
    }

    Ok(())
}
