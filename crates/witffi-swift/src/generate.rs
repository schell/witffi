//! Swift bindings code generator.
//!
//! Placeholder — will be implemented after the Rust generator is validated.

use anyhow::bail;
use wit_parser::{Resolve, WorldId};

/// Configuration for the Swift generator.
#[derive(Debug, Clone)]
pub struct SwiftConfig {
    /// Module name for the generated Swift code.
    pub module_name: String,
}

impl Default for SwiftConfig {
    fn default() -> Self {
        Self {
            module_name: "WitFFI".to_string(),
        }
    }
}

/// Generates Swift bindings from a resolved WIT world.
pub struct SwiftGenerator<'a> {
    resolve: &'a Resolve,
    world_id: WorldId,
    config: SwiftConfig,
}

impl<'a> SwiftGenerator<'a> {
    pub fn new(resolve: &'a Resolve, world_id: WorldId, config: SwiftConfig) -> Self {
        Self {
            resolve,
            world_id,
            config,
        }
    }

    /// Generate Swift bindings code.
    pub fn generate(&self) -> anyhow::Result<String> {
        bail!("Swift generator not yet implemented")
    }
}
