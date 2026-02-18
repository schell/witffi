//! # witffi-core
//!
//! Core WIT parsing and FFI type mapping for the `witffi` code generator.
//!
//! This crate provides:
//! - WIT file loading and resolution via [`load_wit`]
//! - Name conversion utilities for mapping WIT kebab-case identifiers
//!   to language-specific naming conventions
//! - Type analysis helpers for determining FFI characteristics of WIT types

pub mod names;

use std::path::Path;

use anyhow::{bail, Context};
pub use wit_parser;
use wit_parser::{Resolve, UnresolvedPackageGroup, WorldId};

/// Load and resolve WIT definitions from a directory or single file.
///
/// Returns the [`Resolve`] containing all resolved types and the
/// [`WorldId`] of the (single) world defined in the package.
///
/// # Errors
///
/// Returns an error if:
/// - The path does not exist or is not readable
/// - The WIT files contain syntax errors
/// - There is not exactly one world defined
pub fn load_wit(path: &Path) -> anyhow::Result<(Resolve, WorldId)> {
    let mut resolve = Resolve::default();

    if path.is_dir() {
        resolve
            .push_dir(path)
            .with_context(|| format!("failed to load WIT directory: {}", path.display()))?;
        let worlds: Vec<WorldId> = resolve.worlds.iter().map(|(id, _)| id).collect();
        if worlds.len() != 1 {
            bail!(
                "expected exactly 1 world in WIT package, found {}",
                worlds.len()
            );
        }
        return Ok((resolve, worlds[0]));
    }

    let group = UnresolvedPackageGroup::parse_file(path)
        .with_context(|| format!("failed to parse WIT file: {}", path.display()))?;

    let pkg_id = resolve
        .push_group(group)
        .with_context(|| "failed to resolve WIT package")?;

    let pkg_data = &resolve.packages[pkg_id];
    let worlds: Vec<WorldId> = pkg_data.worlds.values().copied().collect();
    if worlds.len() != 1 {
        bail!(
            "expected exactly 1 world in WIT package, found {}",
            worlds.len()
        );
    }
    Ok((resolve, worlds[0]))
}

/// Describes a single exported function from a WIT world, fully qualified.
#[derive(Debug, Clone)]
pub struct ExportedFunction {
    /// The interface name this function belongs to (e.g. "parser").
    pub interface_name: String,
    /// The function name (e.g. "parse").
    pub function_name: String,
    /// The WIT function definition.
    pub function: wit_parser::Function,
}

/// Extract all exported functions from a world.
pub fn exported_functions(resolve: &Resolve, world_id: WorldId) -> Vec<ExportedFunction> {
    let world = &resolve.worlds[world_id];
    let mut result = Vec::new();

    for (key, item) in &world.exports {
        match item {
            wit_parser::WorldItem::Interface { id, .. } => {
                let iface = &resolve.interfaces[*id];
                let iface_name = match key {
                    wit_parser::WorldKey::Name(n) => n.clone(),
                    wit_parser::WorldKey::Interface(id) => resolve.interfaces[*id]
                        .name
                        .clone()
                        .unwrap_or_else(|| format!("interface-{}", id.index())),
                };
                for (_name, func) in &iface.functions {
                    result.push(ExportedFunction {
                        interface_name: iface_name.clone(),
                        function_name: func.name.clone(),
                        function: func.clone(),
                    });
                }
            }
            wit_parser::WorldItem::Function(func) => {
                result.push(ExportedFunction {
                    interface_name: String::new(),
                    function_name: func.name.clone(),
                    function: func.clone(),
                });
            }
            wit_parser::WorldItem::Type { .. } => {
                // Type exports don't produce functions
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_eip681_wit() {
        let wit_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../wit/eip681.wit");
        let (resolve, world_id) = load_wit(&wit_path).expect("failed to load eip681.wit");

        let world = &resolve.worlds[world_id];
        assert_eq!(world.name, "eip681");

        let funcs = exported_functions(&resolve, world_id);
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].interface_name, "parser");
        assert_eq!(funcs[0].function_name, "parse");
    }
}
