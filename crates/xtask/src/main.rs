//! xtask CLI — generate eip681 FFI bindings from WIT definitions.
//!
//! Run via `cargo xtask generate` to regenerate all binding artifacts.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use snafu::prelude::*;

type Result<T, E = snafu::Whatever> = std::result::Result<T, E>;

#[derive(Parser)]
#[command(
    name = "xtask",
    about = "Build task runner for eip681 FFI binding generation"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate all FFI bindings (Rust, C headers, Kotlin, Swift).
    Generate,
}

#[snafu::report]
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate => {
            let workspace_root = workspace_root()?;
            xtask::generate(&workspace_root).whatever_context("binding generation failed")?;
            eprintln!("Done.");
        }
    }

    Ok(())
}

/// Resolve the workspace root from the `CARGO_WORKSPACE_DIR` environment
/// variable.
fn workspace_root() -> Result<PathBuf> {
    let dir = std::env::var("CARGO_WORKSPACE_DIR")
        .whatever_context("CARGO_WORKSPACE_DIR not set — run via `cargo xtask`")?;
    Ok(PathBuf::from(dir))
}
