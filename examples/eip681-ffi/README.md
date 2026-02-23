# eip681-ffi

A Rust crate that exposes the [`eip681`](https://github.com/zcash/librustzcash/tree/main/components/eip681)
parser as a C-compatible FFI library using `witffi`.

This is the **producer** side of the witffi workflow: you start with a WIT
definition, generate Rust FFI scaffolding, implement a trait to bridge your
domain types, and get a shared/static library with stable C symbols.

## How it works

The pipeline has four steps:

1. **Define the interface** in [`wit/eip681.wit`](../../wit/eip681.wit) using
   WIT (WebAssembly Interface Types) as a pure IDL — no WASM runtime involved.

2. **`build.rs` generates code** at build time using the `witffi-core` and
   `witffi-rust` library APIs. It produces three files:
   - `src/ffi.rs` — `#[repr(C)]` structs, a trait, free functions, and a
     `witffi_register!` macro
   - `ffi.h` — a C header declaring all types and exported functions
   - `witffi_types.h` — shared FFI types header (copied from `witffi-types`)

3. **`src/lib.rs` implements the trait** by converting between the `eip681`
   crate's domain types (`TransactionRequest`, `NativeRequest`, etc.) and the
   generated FFI types (`FfiTransactionRequest`, `FfiNativeRequest`, etc.).

4. **`witffi_register!(Impl)`** stamps out all `extern "C"` functions with
   parameter conversion, panic catching, and thread-local error storage.

## Key files

| File | Role |
|------|------|
| [`build.rs`](build.rs) | Runs witffi code generation at build time |
| [`src/lib.rs`](src/lib.rs) | Trait implementation bridging eip681 to FFI types |
| [`src/ffi.rs`](src/ffi.rs) | **Generated** — `repr(C)` types, trait, macro |
| [`ffi.h`](ffi.h) | **Generated** — C header for all exported symbols |
| [`witffi_types.h`](witffi_types.h) | **Generated** — Shared FFI types (`FfiByteSlice`, `FfiByteBuffer`) |

## Building

```sh
# From the witffi repo root:
cargo build -p eip681-ffi            # Debug build
cargo build -p eip681-ffi --release  # Release build (for linking from Swift/Go/etc.)
```

This produces:
- `target/release/libeip681_ffi.a` (static library)
- `target/release/libeip681_ffi.dylib` (dynamic library, macOS)

## Testing

```sh
cargo test -p eip681-ffi
```

Three integration tests verify parsing at the FFI layer: native ETH transfer,
ERC-20 token transfer, and invalid input error handling.

## Seeing the expanded C and Kotlin linkage code 

The `eip681-ffi`'s `lib.rs` file calls two macros `witffi_register_ffi!` and `witffi_register_jni`.
These macros stamp out the linkage between FFI and the `Eip681` trait (generated from the `.wit` file).
To inspect the linkage code, use:

```
cargo expand -p eip681-ffi
```

## Exported C API

The library exports 8 C-compatible symbols (see [`ffi.h`](ffi.h) for full
declarations):

| Function | Purpose |
|----------|---------|
| `zcash_eip681_parser_parse` | Parse an EIP-681 URI string |
| `zcash_eip681_last_error_length` | Get last error message length |
| `zcash_eip681_error_message_utf8` | Copy last error into a buffer |
| `zcash_eip681_clear_last_error` | Clear error state |
| `zcash_eip681_free_byte_buffer` | Free an `FfiByteBuffer` |
| `zcash_eip681_free_native_request` | Free an `FfiNativeRequest` |
| `zcash_eip681_free_erc20_request` | Free an `FfiErc20Request` |
| `zcash_eip681_free_transaction_request` | Free an `FfiTransactionRequest` |

Error convention: `zcash_eip681_parser_parse` returns `NULL` on error. Call
`zcash_eip681_last_error_length()` and `zcash_eip681_error_message_utf8()` to
retrieve the error string.

## Kotlin/JNI support

This crate also generates Kotlin/JNI bindings alongside the C FFI. The
`build.rs` produces:

- `witffi_register_jni!(Impl)` macro in `src/ffi.rs` — JNI entry points that
  convert between Rust and Java types via `jni` crate v0.21
- `../eip681-kotlin/src/Bindings.kt` — Kotlin model classes and a JNI bridge
  class

See [`../eip681-kotlin/`](../eip681-kotlin/) for a self-contained demo that
parses EIP-681 URIs from Kotlin via JNI.

## Adapting for your own crate

To expose a different Rust library as a C FFI using witffi:

1. Write a `.wit` file describing your interface (types, functions, error
   handling via `result<T, E>`)
2. Create a new crate with `crate-type = ["cdylib", "staticlib"]`
3. Add `witffi-core` and `witffi-rust` as build-dependencies, and
   `witffi-types` as a regular dependency
4. Write a `build.rs` modeled on this example's — set `c_prefix` and
   `c_type_prefix` to match your naming convention
5. Implement the generated trait, converting your domain types to the FFI types
6. Call `witffi_register!(YourImpl)` to stamp out the `extern "C"` functions
7. Optionally add `witffi-kotlin` and `jni` dependencies and call
   `witffi_register_jni!(YourImpl)` for Kotlin/Android support
