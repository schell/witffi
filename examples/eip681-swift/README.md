# eip681-swift

A Swift package that consumes the [`eip681-ffi`](../eip681-ffi/) Rust library
via witffi-generated Swift bindings.

This is the **consumer** side of the witffi workflow: the Rust library exports C
symbols, witffi generates idiomatic Swift wrappers, and this package ties it all
together as a standard Swift Package Manager project.

## Prerequisites

- **Rust toolchain** — [rustup.rs](https://rustup.rs)
- **Swift toolchain** — Xcode or [swift.org](https://swift.org) (Swift 6.0+)

## Quick start

```sh
cd examples/eip681-swift

./build.sh          # Build the Rust library + Swift package
./build.sh run      # Build and run the example CLI
./build.sh test     # Build and run the Swift tests
```

### Example output

```
Parsing native transfer:
  URI: ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000
  Result: Native ETH transfer
    Schema:    ethereum
    Recipient: 0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359
    Chain ID:  (default / mainnet)
    Value:     32 bytes (big-endian u256)
    Display:   ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000

Parsing ERC-20 transfer:
  URI: ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/transfer?address=...&uint256=1000000
  Result: ERC-20 token transfer
    Token:     0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
    Recipient: 0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359
    ...

Parsing invalid input:
  URI: not-a-valid-uri
  Error (expected): Parsing error: ...
```

## What `build.sh` does

The script runs three steps before building the Swift package:

1. **`cargo build --release -p eip681-ffi`** — compiles the Rust FFI library,
   producing `target/release/libeip681_ffi.a`
2. **`witffi generate --lang swift`** — generates `Sources/Eip681/Bindings.swift`
   with idiomatic Swift types and wrapper functions
3. **Copies C headers** (`ffi.h`, `witffi_types.h`) into
   `Sources/CZcashEip681/include/` so Swift can import the C module

Then it runs `swift build`, `swift test`, or `swift run` depending on the
argument.

## Package structure

```
eip681-swift/
├── Package.swift
├── build.sh
├── Sources/
│   ├── CZcashEip681/              # C target: headers + module map
│   │   ├── include/
│   │   │   ├── module.modulemap   # Clang module wrapping ffi.h + witffi_types.h
│   │   │   ├── ffi.h              # Generated C header (copied by build.sh)
│   │   │   └── witffi_types.h     # Shared FFI types (copied by build.sh)
│   │   └── shim.c                 # Empty file (SPM requires one source file)
│   ├── Eip681/                    # Swift library target
│   │   └── Bindings.swift         # Generated Swift bindings (witffi output)
│   └── eip681-example/            # Executable target
│       └── main.swift             # CLI demo: parse URIs, print results
└── Tests/
    └── Eip681Tests/
        └── Eip681Tests.swift      # 4 tests using Swift Testing framework
```

### Why four targets?

| Target | Type | Purpose |
|--------|------|---------|
| `CZcashEip681` | C library | Exposes the FFI headers to Swift via a `module.modulemap` |
| `Eip681` | Swift library | Contains the generated bindings; links `libeip681_ffi.a` |
| `eip681-example` | Executable | CLI demo that exercises the API |
| `Eip681Tests` | Test | Verifies parsing, error handling, and round-tripping |

## Generated Swift API

The generated `Bindings.swift` provides native Swift types and a throwing API:

```swift
// Value types — no pointers, no manual memory management
public struct NativeRequest {
    public let schemaPrefix: String
    public let chainId: UInt64?
    public let recipientAddress: String
    public let valueAtomic: Data?
    public let gasLimit: Data?
    public let gasPrice: Data?
    public let display: String
}

public struct Erc20Request { ... }

public enum TransactionRequest {
    case native(NativeRequest)
    case erc20(Erc20Request)
    case unrecognised(String)
}

// Namespace with static methods
public enum Eip681 {
    public static func parserParse(_ input: String) throws -> TransactionRequest
}
```

All FFI memory is copied into Swift-native types (`String`, `Data`, `UInt64?`)
and freed immediately — the public API has no pointer management.

## Adapting for your own project

To create a Swift package consuming a different witffi-generated library:

1. **Build the Rust FFI library** — see [`eip681-ffi`](../eip681-ffi/) for the
   producer-side setup
2. **Generate Swift bindings** — run `witffi generate --lang swift --c-prefix your_prefix`
3. **Create a C target** with a `module.modulemap` wrapping your generated
   `ffi.h` and `witffi_types.h`
4. **Create a Swift library target** that depends on the C target and links your
   Rust `.a` via `linkerSettings: [.unsafeFlags(["-L/path/to/lib", "-lyour_lib"])]`
5. **Use the generated API** — `import YourModule` and call the static methods

The `module.modulemap` follows a standard pattern:

```
module CYourModule {
    header "ffi.h"
    header "witffi_types.h"
    export *
}
```
