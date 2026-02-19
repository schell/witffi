//! EIP-681 FFI library — a complete example of using `witffi`.
//!
//! This crate demonstrates how to use `witffi` to expose a Rust library as a
//! C-compatible FFI. It bridges the [`eip681`] crate's high-level API to the
//! generated `repr(C)` types, producing a shared/static library that C, Swift,
//! Go, or Kotlin callers can link against.
//!
//! ## How it works
//!
//! 1. `build.rs` runs the `witffi` code generator against `wit/eip681.wit`
//! 2. The generated code (included below) defines `repr(C)` FFI types, a trait,
//!    and a `witffi_register!` macro
//! 3. This file implements the trait, converting between `eip681` domain types
//!    and the FFI types
//! 4. `witffi_register!(Impl)` stamps out the actual `extern "C"` symbols
#![allow(non_camel_case_types, non_snake_case, unused_unsafe)]

// Pull in the generated FFI types, trait, and macro.
include!(concat!(env!("OUT_DIR"), "/ffi.rs"));

use std::ptr;

use witffi_types::FfiByteBuffer;

// ---- Conversion helpers ----

/// Convert a `U256` into an `FfiByteBuffer` containing 32 bytes big-endian.
fn u256_to_ffi(v: eip681::U256) -> FfiByteBuffer {
    let mut bytes = vec![0u8; 32];
    v.to_big_endian(&mut bytes);
    FfiByteBuffer::from_vec(bytes)
}

/// Convert an `eip681::NativeRequest` into its FFI representation.
fn native_to_ffi(r: &eip681::NativeRequest) -> FfiNativeRequest {
    FfiNativeRequest {
        schema_prefix: FfiByteBuffer::from_string(r.schema_prefix().to_string()),
        chain_id: witffi_types::option_to_ptr(r.chain_id()),
        recipient_address: FfiByteBuffer::from_string(r.recipient_address().to_string()),
        value_atomic: witffi_types::option_to_ptr(r.value_atomic().map(u256_to_ffi)),
        gas_limit: witffi_types::option_to_ptr(r.gas_limit().map(u256_to_ffi)),
        gas_price: witffi_types::option_to_ptr(r.gas_price().map(u256_to_ffi)),
        display: FfiByteBuffer::from_string(format!("{r}")),
    }
}

/// Convert an `eip681::Erc20Request` into its FFI representation.
fn erc20_to_ffi(r: &eip681::Erc20Request) -> FfiErc20Request {
    FfiErc20Request {
        chain_id: witffi_types::option_to_ptr(r.chain_id()),
        token_contract_address: FfiByteBuffer::from_string(r.token_contract_address().to_string()),
        recipient_address: FfiByteBuffer::from_string(r.recipient_address().to_string()),
        value_atomic: u256_to_ffi(r.value_atomic()),
        display: FfiByteBuffer::from_string(format!("{r}")),
    }
}

/// Convert an `eip681::TransactionRequest` into its FFI representation.
fn tx_request_to_ffi(r: eip681::TransactionRequest) -> FfiTransactionRequest {
    match r {
        eip681::TransactionRequest::NativeRequest(ref native) => FfiTransactionRequest {
            tag: FfiTransactionRequestTag::Native,
            native: Box::into_raw(Box::new(FfiTransactionRequestNativePayload {
                value: native_to_ffi(native),
            })),
            erc20: ptr::null_mut(),
            unrecognised: ptr::null_mut(),
        },
        eip681::TransactionRequest::Erc20Request(ref erc20) => FfiTransactionRequest {
            tag: FfiTransactionRequestTag::Erc20,
            native: ptr::null_mut(),
            erc20: Box::into_raw(Box::new(FfiTransactionRequestErc20Payload {
                value: erc20_to_ffi(erc20),
            })),
            unrecognised: ptr::null_mut(),
        },
        eip681::TransactionRequest::Unrecognised(raw) => FfiTransactionRequest {
            tag: FfiTransactionRequestTag::Unrecognised,
            native: ptr::null_mut(),
            erc20: ptr::null_mut(),
            unrecognised: Box::into_raw(Box::new(FfiTransactionRequestUnrecognisedPayload {
                value: FfiByteBuffer::from_string(format!("{raw}")),
            })),
        },
    }
}

// ---- Trait implementation ----

/// The concrete implementation that bridges `eip681` to the generated FFI trait.
struct Impl;

impl Eip681 for Impl {
    fn parser_parse(input: &str) -> Result<FfiTransactionRequest, String> {
        eip681::TransactionRequest::parse(input)
            .map(tx_request_to_ffi)
            .map_err(|e| format!("{e}"))
    }
}

// Stamp out all `extern "C"` FFI functions.
witffi_register!(Impl);

// ---- Tests ----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_native_transfer() {
        let input = "ethereum:0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359?value=2014000000000000000";
        let result = <Impl as Eip681>::parser_parse(input);
        assert!(result.is_ok(), "parse failed: {:?}", result.err());

        let ffi = result.unwrap();
        assert_eq!(ffi.tag, FfiTransactionRequestTag::Native);
        assert!(!ffi.native.is_null());
        assert!(ffi.erc20.is_null());
        assert!(ffi.unrecognised.is_null());

        // Read the native payload
        let payload = unsafe { &*ffi.native };
        let addr_slice = unsafe {
            std::slice::from_raw_parts(
                payload.value.recipient_address.ptr,
                payload.value.recipient_address.len,
            )
        };
        let addr = std::str::from_utf8(addr_slice).unwrap();
        assert_eq!(addr, "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359");

        // chain_id should be None (null)
        assert!(payload.value.chain_id.is_null());

        // value_atomic should be Some (non-null)
        assert!(!payload.value.value_atomic.is_null());

        // Clean up (in real code, the C caller would call the free functions)
    }

    #[test]
    fn test_parse_erc20_transfer() {
        let input = "ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48/transfer?address=0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359&uint256=1000000";
        let result = <Impl as Eip681>::parser_parse(input);
        assert!(result.is_ok(), "parse failed: {:?}", result.err());

        let ffi = result.unwrap();
        assert_eq!(ffi.tag, FfiTransactionRequestTag::Erc20);
        assert!(ffi.native.is_null());
        assert!(!ffi.erc20.is_null());
        assert!(ffi.unrecognised.is_null());
    }

    #[test]
    fn test_parse_invalid_input() {
        let input = "not-a-valid-uri";
        let result = <Impl as Eip681>::parser_parse(input);
        assert!(result.is_err());
    }
}
