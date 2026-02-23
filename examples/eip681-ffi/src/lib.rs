//! EIP-681 FFI library — a complete example of using `witffi`.
//!
//! This crate demonstrates how to use `witffi` to expose a Rust library as a
//! C-compatible FFI and/or JNI library. It bridges the [`eip681`] crate's
//! high-level API to the generated idiomatic Rust types, then uses the
//! registration macros to produce the actual FFI/JNI symbols.
//!
//! ## How it works
//!
//! 1. `build.rs` runs the `witffi` code generator against `wit/eip681.wit`
//! 2. The generated code (included below) defines idiomatic Rust types, a trait,
//!    and `witffi_register_ffi!` / `witffi_register_jni!` macros
//! 3. This file implements the trait, converting between `eip681` domain types
//!    and the generated idiomatic types
//! 4. `witffi_register_ffi!(Impl)` stamps out the `extern "C"` symbols (C-ABI)
//! 5. `witffi_register_jni!(Impl)` stamps out `Java_` JNI entry points
#![allow(non_camel_case_types, non_snake_case, unused_unsafe)]

// build.rs generates src/ffi.rs — pull it in as a module.
mod ffi;
use ffi::*;

pub use ffi::U256;

// ---- Conversion helpers ----

/// Convert an `eip681::NativeRequest` into the generated idiomatic `NativeRequest`.
fn native_to_idiomatic(r: &eip681::NativeRequest) -> NativeRequest {
    NativeRequest {
        schema_prefix: r.schema_prefix().to_string(),
        chain_id: r.chain_id(),
        recipient_address: r.recipient_address().to_string(),
        value_atomic: r.value_atomic().map(u256_to_bytes),
        gas_limit: r.gas_limit().map(u256_to_bytes),
        gas_price: r.gas_price().map(u256_to_bytes),
        display: format!("{r}"),
    }
}

/// Convert an `eip681::Erc20Request` into the generated idiomatic `Erc20Request`.
fn erc20_to_idiomatic(r: &eip681::Erc20Request) -> Erc20Request {
    Erc20Request {
        chain_id: r.chain_id(),
        token_contract_address: r.token_contract_address().to_string(),
        recipient_address: r.recipient_address().to_string(),
        value_atomic: u256_to_bytes(r.value_atomic()),
        display: format!("{r}"),
    }
}

/// Convert a `U256` into a 32-byte big-endian `Vec<u8>`.
fn u256_to_bytes(v: eip681::U256) -> Vec<u8> {
    let mut bytes = vec![0u8; 32];
    v.to_big_endian(&mut bytes);
    bytes
}

/// Convert an `eip681::TransactionRequest` into the generated idiomatic type.
fn tx_request_to_idiomatic(r: eip681::TransactionRequest) -> TransactionRequest {
    match r {
        eip681::TransactionRequest::NativeRequest(ref native) => {
            TransactionRequest::Native(native_to_idiomatic(native))
        }
        eip681::TransactionRequest::Erc20Request(ref erc20) => {
            TransactionRequest::Erc20(erc20_to_idiomatic(erc20))
        }
        eip681::TransactionRequest::Unrecognised(raw) => {
            TransactionRequest::Unrecognised(format!("{raw}"))
        }
    }
}

// ---- Trait implementation ----

/// The concrete implementation that bridges `eip681` to the generated trait.
struct Impl;

impl Eip681 for Impl {
    fn parser_parse(input: &str) -> Result<TransactionRequest, String> {
        eip681::TransactionRequest::parse(input)
            .map(tx_request_to_idiomatic)
            .map_err(|e| format!("{e}"))
    }

    fn functions_u256_to_string(input: &[u8]) -> String {
        let u256 = eip681::U256::from_big_endian(input);
        format!("{u256}")
    }
}

// Stamp out `extern "C"` FFI functions (for Swift, Go, C consumers).
witffi_register!(Impl);

// Stamp out `Java_` JNI entry points (for Kotlin/Android consumers).
witffi_register_jni!(Impl);

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_native_transfer() {
        let input = "ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000";
        let result = <Impl as Eip681>::parser_parse(input);
        assert!(result.is_ok(), "parse failed: {:?}", result.err());

        let tx = result.unwrap();
        match tx {
            TransactionRequest::Native(native) => {
                assert_eq!(
                    native.recipient_address,
                    "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359"
                );
                assert!(native.chain_id.is_none());
                assert!(native.value_atomic.is_some());
            }
            other => panic!("expected Native variant, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_erc20_transfer() {
        let input = "ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/transfer?address=0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359&uint256=1000000";
        let result = <Impl as Eip681>::parser_parse(input);
        assert!(result.is_ok(), "parse failed: {:?}", result.err());

        let tx = result.unwrap();
        match tx {
            TransactionRequest::Erc20(erc20) => {
                assert_eq!(
                    erc20.token_contract_address,
                    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
                );
                assert_eq!(
                    erc20.recipient_address,
                    "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359"
                );
            }
            other => panic!("expected Erc20 variant, got: {other:?}"),
        }
    }

    #[test]
    fn test_parse_invalid_input() {
        let input = "not-a-valid-uri";
        let result = <Impl as Eip681>::parser_parse(input);
        assert!(result.is_err());
    }
}
