# witffi

Generate native FFI bindings from [WIT](https://component-model.bytecodealliance.org/design/wit.html) definitions.

`witffi` takes `.wit` files as a high-level interface description language and produces
C-ABI-compatible scaffolding code for cross-language interop. Rather than targeting the
WebAssembly component model, it uses WIT purely as an IDL for **native** FFI.

> **Status:** Early stage (v0.1.0). The Rust/C generator is functional. Swift generation is planned.

## Supported Targets

| Language | Output | Status |
|----------|--------|--------|
| Rust + C | `ffi.rs` + `ffi.h` | Working |
| Swift | `Bindings.swift` | Planned |

## What Gets Generated

For a given WIT world, the Rust generator produces:

- **`#[repr(C)]` types** — records become C-layout structs, variants become tagged unions, enums become `#[repr(u32)]`, flags become `u32` bitfields
- **An implementation trait** — one method per exported function, using idiomatic Rust types (`&str`, `Option<T>`, `Result<T, E>`, etc.)
- **`extern "C"` wrapper functions** — with `catch_unwind` for panic safety and thread-local error storage
- **Free functions** — `free_byte_buffer()` and `free_<type>()` for every heap-allocated return type
- **Error handling** — `_last_error_length()`, `_error_message_utf8()`, `_clear_last_error()` following the Mozilla/UniFFI pattern
- **A C header** — complete `.h` with typedefs, enum definitions, and function declarations

## Project Structure

```
witffi/
├── Cargo.toml              # Workspace root
├── wit/
│   └── eip681.wit          # Example WIT definition
└── crates/
    ├── witffi-core/        # WIT loading, name conventions, type analysis
    ├── witffi-rust/        # Rust + C header code generator
    ├── witffi-swift/       # Swift bindings generator (stub)
    └── witffi-cli/         # CLI binary
```

## Installation

Build from source (requires Rust 2024 edition / rustc nightly or recent stable):

```sh
cargo install --path crates/witffi-cli
```

Or build locally:

```sh
cargo build --release
```

The binary is named `witffi`.

## Usage

```sh
witffi generate --wit <WIT_PATH> --lang <LANGUAGE> --output <DIR> [OPTIONS]
```

### Arguments

| Flag | Short | Description | Default |
|------|-------|-------------|---------|
| `--wit` | `-w` | Path to a `.wit` file or directory | required |
| `--lang` | `-l` | Target language (`rust` or `swift`) | required |
| `--output` | `-o` | Output directory for generated files | required |
| `--c-prefix` | | Prefix for C function names | `witffi` |
| `--c-type-prefix` | | Prefix for C type names | `Ffi` |

### Example

Given a WIT file like `wit/eip681.wit`:

```wit
package zcash:eip681;

interface types {
    type u256 = list<u8>;

    record native-request {
        chain-id: option<u64>,
        recipient-address: string,
        value-atomic: option<u256>,
    }

    variant transaction-request {
        native(native-request),
        unrecognised(string),
    }
}

interface parser {
    use types.{transaction-request};
    parse: func(input: string) -> result<transaction-request, string>;
}

world eip681 {
    export parser;
}
```

Generate Rust scaffolding and a C header:

```sh
witffi generate \
  --wit wit/eip681.wit \
  --lang rust \
  --output out/ \
  --c-prefix zcash_eip681
```

This produces:
- `out/ffi.rs` — Rust scaffolding with `#[repr(C)]` types, a `trait Eip681`, and `extern "C"` wrappers
- `out/ffi.h` — Corresponding C header

## Workflow

1. **Define** your library's public API in a `.wit` file
2. **Generate** FFI scaffolding with `witffi generate`
3. **Implement** the generated Rust trait in your library
4. **Compile** your library as a C-compatible dynamic or static library
5. **Generate** FFI scaffolding for the host (Swift, etc)
6. **Call** from Swift, C, or any language that supports the C ABI using the generated header

## Running Tests

```sh
cargo test
```

Tests cover WIT loading, name convention mapping, and end-to-end code generation
against the included `eip681.wit` example.

## Examples

See the [examples](examples/) directory. 

## License

Licensed under either of

- [MIT license](http://opensource.org/licenses/MIT)
- [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)

at your option.
