# eip681-kotlin

A self-contained Kotlin/JNI example that calls the Rust `eip681` parser
through the `witffi`-generated JNI bridge.

This is the **consumer** side of the witffi Kotlin workflow: the generated
`Bindings.kt` provides typed Kotlin classes and `external fun` declarations
that map to the Rust JNI symbols produced by `witffi_register_jni!`.

## How it works

```
                 witffi                           witffi
  eip681.wit ──────────> ffi.rs (Rust JNI)   ──────────> Bindings.kt (Kotlin)
                         witffi_register_jni!              sealed classes
                         Java_* entry points               external fun
                              │                                 │
                              ▼                                 ▼
                     libeip681_ffi.dylib  <──── JNI ────  Main.kt (this demo)
```

1. **`eip681-ffi/build.rs`** generates both `ffi.rs` (Rust scaffolding with
   `witffi_register_jni!`) and `Bindings.kt` (Kotlin model classes + JNI
   declarations) from the same WIT definition.

2. **`eip681-ffi/src/lib.rs`** implements the `Eip681` trait and invokes both
   `witffi_register!(Impl)` (C-ABI) and `witffi_register_jni!(Impl)` (JNI).

3. **`Bindings.kt`** (generated, in `src/`) defines `NativeRequest`,
   `Erc20Request`, `TransactionRequest` (sealed class), and the `Eip681`
   implementation class with `System.loadLibrary()` and `external fun`
   declarations.

4. **`Main.kt`** (hand-written) creates an `Eip681()` instance and calls
   `parserParse()` with various URIs, demonstrating the full Rust-to-Kotlin
   round trip.

## Prerequisites

- **Rust** (with `cargo`)
- **Kotlin** (`kotlinc` — `brew install kotlin`)
- **Java** (JDK 17+ — `brew install openjdk`)

## Quick start

```sh
# From this directory:
make
```

This will:
1. Build the Rust `eip681-ffi` crate (produces `libeip681_ffi.dylib`)
2. Compile `Bindings.kt` + `Main.kt` into `demo.jar`
3. Run the demo with `java -Djava.library.path=... -jar demo.jar`

## Expected output

```
=== EIP-681 Kotlin/JNI Demo ===

Parsing: ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000
  Type:      Native ETH transfer
  Schema:    ethereum
  To:        0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359
  Chain ID:  mainnet (default)
  Value:     32 bytes (big-endian)
  Display:   ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000

Parsing: ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/transfer?address=...&uint256=1000000
  Type:      ERC-20 token transfer
  Token:     0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
  To:        0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359
  Chain ID:  mainnet (default)
  Amount:    32 bytes (big-endian)
  Display:   ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/transfer?...

Parsing: not-a-valid-uri
  Caught expected error: ...

Done.
```

## Key files

| File | Role |
|------|------|
| [`src/Bindings.kt`](src/Bindings.kt) | **Generated** — Kotlin model classes + JNI bridge |
| [`src/Main.kt`](src/Main.kt) | Hand-written demo application |
| [`src/Keep.kt`](src/Keep.kt) | Stub `@Keep` annotation (on Android this comes from `androidx`) |
| [`Makefile`](Makefile) | Build + run instructions |

## Adapting for Android

On Android, replace `src/Keep.kt` with the real `androidx.annotation.Keep`
dependency. The generated `Bindings.kt` works identically — it uses
`System.loadLibrary()` which is the standard Android NDK mechanism.

Build the Rust crate as a `cdylib` targeting Android ABIs (`aarch64-linux-android`,
`armv7-linux-androideabi`, `x86_64-linux-android`) using
[`rust-android-gradle`](https://github.com/nickelc/rust-android-gradle) or
cross-compilation with the Android NDK.
