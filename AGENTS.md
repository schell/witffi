# AGENTS.md

Coding agent instructions for the `witffi` repository.

## Build / Lint / Test Commands

```sh
cargo build                              # Debug build (all crates)
cargo build --release                    # Release build
cargo fmt --all                          # Format all crates
cargo fmt --all -- --check               # Check formatting without modifying
cargo clippy --workspace                 # Lint all crates
cargo test                               # Run all tests
cargo test -p witffi-core                # Run tests for a single crate
cargo test -p witffi-rust                # Run tests for the Rust generator
cargo test test_load_eip681_wit          # Run a single test by name
cargo test -p witffi-core test_rust_type # Run a single test in a specific crate (substring match)
```

Run the CLI locally:

```sh
cargo run -p witffi-cli -- generate --wit wit/eip681.wit --lang rust --output out/
```

## Project Architecture

Workspace with 4 crates. Dependency flow: `witffi-cli` -> `{witffi-core, witffi-rust, witffi-swift}`, and `witffi-rust`/`witffi-swift` -> `witffi-core`.

| Crate | Role |
|-------|------|
| `witffi-core` | WIT loading (`load_wit`), name convention mapping, type analysis helpers |
| `witffi-rust` | Rust `extern "C"` scaffolding + C header generator (~1200 lines) |
| `witffi-swift` | Swift bindings generator (stub, not yet implemented) |
| `witffi-cli` | `witffi` binary, thin clap wrapper that wires the other crates together |

All dependencies are declared at the workspace level in the root `Cargo.toml` and referenced with `.workspace = true` in crate manifests. The WIT fixture at `wit/eip681.wit` is used by tests across crates.

## Rust Edition

This project uses **Rust edition 2024**. Key implications for generated and hand-written code:

- Use `#[unsafe(no_mangle)]` instead of `#[no_mangle]`
- Use `const` blocks in `thread_local!`: `= const { RefCell::new(None) };`
- `unsafe extern "C" fn` syntax (the `unsafe` qualifier is on the `extern` block)

## Code Style

### Formatting

- **4 spaces** indentation, no tabs
- **Default `rustfmt`** — no `rustfmt.toml` exists; do not create one
- **Default `clippy`** — no `clippy.toml` exists; do not create one
- **Trailing commas** on struct fields, enum variants, match arms, and multi-line arguments
- **K&R brace style** — opening brace on the same line as the declaration
- Break method chains one-per-line with leading `.` indented once

### Imports

Organize imports in three groups separated by blank lines:

```rust
use std::path::Path;                          // 1. std

use anyhow::{bail, Context};                  // 2. External crates
use wit_parser::{Resolve, WorldId};

use witffi_core::{exported_functions, names}; // 3. Internal/workspace crates
```

- Use **absolute crate names** for cross-crate imports (`use witffi_core::...`)
- Use **`use super::*`** only inside `#[cfg(test)] mod tests`
- Group multiple items from one module with braces: `use wit_parser::{Resolve, Type, WorldId};`
- Keep separate `use` lines for different `std` submodules (don't merge into `use std::{...}`)
- Import traits for method access without the name: `use anyhow::Context as _;`

### Re-exports

Crate `lib.rs` files re-export the primary type as a facade:

```rust
pub use generate::RustGenerator;
```

So consumers write `witffi_rust::RustGenerator` instead of `witffi_rust::generate::RustGenerator`.

### Error Handling

- **`anyhow` only** — no `thiserror`, no custom error types
- Return `anyhow::Result<T>` from all fallible functions
- Use `bail!("message")` for precondition failures
- Use `.with_context(|| format!("doing X: {}", detail))?` to annotate errors from called functions
- Context messages are **lowercase** descriptive clauses: `"failed to load WIT directory: ..."`
- Use `.expect("reason")` only in tests, never in library/CLI code

### Documentation

- **`//!` module docs** on every `.rs` file — describe what the module provides
- **`///` doc comments** on all public functions, types, traits, and struct fields
- Include `# Errors` section in doc comments for fallible public functions
- Include `# Safety` section for any unsafe functions
- Private helpers generally have no doc comments
- Use `// ---- Section Name ----` comments to divide large files into logical sections

### Naming

| Item | Convention | Example |
|------|-----------|---------|
| Functions | `snake_case` | `load_wit`, `exported_functions` |
| Types / Traits | `PascalCase` | `RustGenerator`, `ExportedFunction` |
| Modules | `snake_case` | `names`, `generate` |
| Crate names | `kebab-case` | `witffi-core`, `witffi-rust` |
| Test functions | `test_` prefix + `snake_case` | `test_load_eip681_wit` |
| Enum variants | `PascalCase` | `Language::Rust` |
| Constants | `SHOUTY_SNAKE_CASE` | (used in generated code) |

### Visibility

- Public structs have **`pub` fields** — no getter/setter pattern
- Private helper functions use default visibility (no `pub`)
- Use `pub mod` only for modules that need external access

### Derives and Types

- Minimum derives on data structs: `#[derive(Debug, Clone)]`
- **Manual `Default` impl** for config structs (not `#[derive(Default)]`)
- Generators hold borrowed data: `pub struct RustGenerator<'a>` with `resolve: &'a Resolve`
- No builder pattern — plain struct construction

## Generator Pattern

Both `RustGenerator` and `SwiftGenerator` follow the same shape:

```rust
pub struct FooGenerator<'a> {
    resolve: &'a Resolve,
    world_id: WorldId,
    config: FooConfig,
}

impl<'a> FooGenerator<'a> {
    pub fn new(resolve: &'a Resolve, world_id: WorldId, config: FooConfig) -> Self { ... }
    pub fn generate(&self) -> anyhow::Result<String> { ... }
}
```

Code generation uses `writeln!(out, ...)` into a `String` via the `std::fmt::Write` trait. The `?` operator propagates `fmt::Error` automatically converted to `anyhow::Error`.

## Testing

- Tests live in `#[cfg(test)] mod tests { use super::*; ... }` at the bottom of each file
- Test names are prefixed with `test_` and are descriptive
- Use `assert_eq!` for value comparisons
- Use `assert!(expr, "message")` for checking generated code contains expected strings
- Load the WIT fixture via `env!("CARGO_MANIFEST_DIR")`:
  ```rust
  let wit_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../wit/eip681.wit");
  ```
- `pretty_assertions` is available as a dev-dependency but not yet used
- Use `eprintln!` for debug output in tests (not `println!` or `dbg!`)
