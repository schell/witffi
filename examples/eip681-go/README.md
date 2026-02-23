# eip681-go

A Go package that consumes the [`eip681-ffi`](../eip681-ffi/) Rust library
via witffi-generated CGo bindings.

This is the **consumer** side of the witffi Go workflow: the Rust library
exports C symbols, witffi generates idiomatic Go wrappers with CGo, and this
package ties it all together as a standard Go module.

## How it works

```
                 witffi                          witffi
  eip681.wit ──────────> ffi.rs (Rust C-ABI) ──────────> bindings.go (Go/CGo)
                         witffi_register_ffi!             Go structs + interfaces
                         extern "C" functions             C.zcash_eip681_*()
                              │                                │
                              v                                v
                     libeip681_ffi.a  <──── CGo link ──── main.go (this demo)
```

1. **`eip681-ffi/build.rs`** generates `ffi.rs` (Rust scaffolding) and C
   headers (`ffi.h`, `witffi_types.h`) from the WIT definition.

2. **`eip681-ffi/src/lib.rs`** implements the `Eip681` trait and invokes
   `witffi_register_ffi!(Impl)` to produce `extern "C"` functions.

3. **`bindings.go`** (generated, in this directory) defines native Go types
   (`NativeRequest`, `Erc20Request`, `TransactionRequest` variant interface)
   and public API functions (`ParserParse`, `FunctionsU256ToString`) that call
   the C-ABI layer via CGo. All FFI memory is deep-copied into Go-native types
   and freed immediately.

4. **`cmd/eip681-example/main.go`** (hand-written) calls the generated API
   with various URIs, demonstrating the full Rust-to-Go round trip.

## Prerequisites

- **Rust** (with `cargo`)
- **Go** 1.20+ (required for `unsafe.StringData` / `unsafe.SliceData`)
- **C compiler** (CGo requires `cc` — Xcode CLI tools on macOS, `gcc` on Linux)

## Quick start

```sh
# From this directory:
make
```

This will:
1. Build the Rust `eip681-ffi` crate (produces `libeip681_ffi.a` + `.dylib`)
2. Build and run the Go demo via `go run`

Other targets:

```sh
make test    # Build Rust and run Go tests
make build   # Build Rust + Go binary without running
make run     # Run the demo (assumes Rust already built)
make clean   # Remove the built Go binary
```

## Expected output

```
=== EIP-681 Go/CGo Demo ===

Parsing: ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000
  Type:      Native ETH transfer
  Schema:    ethereum
  To:        0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359
  Chain ID:  mainnet (default)
  Value:     32 bytes (big-endian u256)
  Display:   ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000

Parsing: ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/transfer?address=...&uint256=1000000
  Type:      ERC-20 token transfer
  Token:     0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
  To:        0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359
  Chain ID:  mainnet (default)
  Amount:    32 bytes (big-endian u256)
  Display:   ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/transfer?...

U256ToString([0 0 ... 42]):
  Result: 42

Parsing: not-a-valid-uri
  Caught expected error: ...

Done.
```

## Package structure

```
eip681-go/
├── go.mod                          # Go module (requires Go 1.20+)
├── bindings.go                     # Generated — Go types + CGo API wrappers
├── bindings_test.go                # Hand-written Go tests (4 tests)
├── ffi.h                           # Generated C header (copied by xtask)
├── witffi_types.h                  # Shared FFI types (copied by xtask)
├── Makefile                        # Build + test + run
└── cmd/
    └── eip681-example/
        └── main.go                 # Hand-written CLI demo
```

## Key files

| File | Role |
|------|------|
| [`bindings.go`](bindings.go) | **Generated** — Go structs, variant interface, CGo API wrappers |
| [`bindings_test.go`](bindings_test.go) | Hand-written Go tests (parse, error, round-trip) |
| [`cmd/eip681-example/main.go`](cmd/eip681-example/main.go) | Hand-written CLI demo |
| [`ffi.h`](ffi.h) | **Generated** — C header for the FFI functions |
| [`witffi_types.h`](witffi_types.h) | **Generated** — `FfiByteSlice` / `FfiByteBuffer` definitions |
| [`Makefile`](Makefile) | Build + test + run instructions |

## Generated Go API

The generated `bindings.go` provides native Go types and an error-returning API:

```go
// Value types — no pointers, no manual memory management
type NativeRequest struct {
    SchemaPrefix     string
    ChainId          *uint64
    RecipientAddress string
    ValueAtomic      []byte
    GasLimit         []byte
    GasPrice         []byte
    Display          string
}

type Erc20Request struct { ... }

// Variant as interface + concrete types
type TransactionRequest = transactionRequestVariant

type TransactionRequestNative struct { Value NativeRequest }
type TransactionRequestErc20 struct { Value Erc20Request }
type TransactionRequestUnrecognised struct { Value string }

// Public API
func ParserParse(input string) (TransactionRequest, error)
func FunctionsU256ToString(input []byte) string
```

All FFI memory is copied into Go-native types (`string`, `[]byte`, `*uint64`)
and freed immediately — the public API has no pointer management.

## Adapting for your own project

To create a Go package consuming a different witffi-generated library:

1. **Build the Rust FFI library** — see [`eip681-ffi`](../eip681-ffi/) for
   the producer-side setup
2. **Generate Go bindings** — run
   `witffi generate --lang go --c-prefix your_prefix --lib-name your_lib`
3. **Copy C headers** (`ffi.h`, `witffi_types.h`) into the Go package directory
   (CGo requires headers alongside `.go` source files)
4. **Set `CGO_LDFLAGS`** to point at the directory containing your Rust library:
   `CGO_LDFLAGS="-L/path/to/lib" go build`
5. **Use the generated API** — `import "your/module/path"` and call the
   generated functions
