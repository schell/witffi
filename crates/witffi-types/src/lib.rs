//! Shared `repr(C)` FFI types for `witffi`-generated code.
//!
//! This crate provides the fundamental types that all `witffi`-generated FFI
//! libraries depend on:
//!
//! - [`FfiByteSlice`]: A borrowed, caller-owned byte slice (const pointer)
//! - [`FfiByteBuffer`]: An owned, callee-allocated byte buffer (must be freed)
//! - [`option_to_ptr`]: Convert `Option<T>` to a nullable heap pointer
//! - [`free_ptr`]: Free a heap-allocated value returned by [`option_to_ptr`]
//!
//! Generated code references these types via fully-qualified paths
//! (e.g. `witffi_types::FfiByteBuffer`) so consumers only need to add
//! `witffi-types` as a dependency.

use std::ptr;

/// An FFI-safe borrowed byte slice (caller-owned, const pointer).
///
/// Used for input parameters: the caller owns the data and the callee
/// must not free it. For strings, the data is UTF-8 encoded without a
/// null terminator.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FfiByteSlice {
    /// Pointer to the first byte (must be valid for `len` bytes, or null if `len == 0`).
    pub ptr: *const u8,
    /// Number of bytes.
    pub len: usize,
}

impl FfiByteSlice {
    /// View the slice as a byte slice.
    ///
    /// # Safety
    ///
    /// The pointer must be valid for `len` bytes and the data must not be
    /// mutated for the lifetime of the returned reference.
    pub unsafe fn as_bytes(&self) -> &[u8] {
        if self.ptr.is_null() || self.len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
        }
    }

    /// View the slice as a `&str` without checking UTF-8 validity.
    ///
    /// # Safety
    ///
    /// The pointer must be valid for `len` bytes of valid UTF-8, and the
    /// data must not be mutated for the lifetime of the returned reference.
    pub unsafe fn as_str_unchecked(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }
}

/// An FFI-safe owned byte buffer (callee-allocated, must be freed).
///
/// Used for output values: the callee allocates the data and the caller
/// is responsible for freeing it via [`FfiByteBuffer::free`] or the
/// generated `*_free_byte_buffer` function.
///
/// For strings, the data is UTF-8 encoded without a null terminator.
/// For byte arrays (e.g. `list<u8>`, `u256`), the data is raw bytes.
#[repr(C)]
#[derive(Debug)]
pub struct FfiByteBuffer {
    /// Pointer to the first byte (must be valid for `len` bytes, or null if empty).
    pub ptr: *mut u8,
    /// Number of bytes.
    pub len: usize,
}

impl FfiByteBuffer {
    /// Create an empty buffer (null pointer, zero length).
    pub fn empty() -> Self {
        Self {
            ptr: ptr::null_mut(),
            len: 0,
        }
    }

    /// Create a buffer from an owned `Vec<u8>`.
    ///
    /// The vector's allocation is transferred to the buffer. The caller
    /// must eventually free it via [`FfiByteBuffer::free`].
    pub fn from_vec(mut v: Vec<u8>) -> Self {
        let buf = Self {
            ptr: v.as_mut_ptr(),
            len: v.len(),
        };
        std::mem::forget(v);
        buf
    }

    /// Create a buffer from an owned `String`.
    ///
    /// The string's UTF-8 bytes are transferred to the buffer. The caller
    /// must eventually free it via [`FfiByteBuffer::free`].
    pub fn from_string(s: String) -> Self {
        Self::from_vec(s.into_bytes())
    }

    /// Free this buffer, deallocating the underlying memory.
    ///
    /// After calling this method, the buffer must not be used again.
    ///
    /// # Safety
    ///
    /// The buffer must have been created by [`FfiByteBuffer::from_vec`],
    /// [`FfiByteBuffer::from_string`], or equivalent allocation from this
    /// library. Calling this on a buffer not allocated by Rust is undefined
    /// behavior.
    pub unsafe fn free(self) {
        if !self.ptr.is_null() && self.len > 0 {
            drop(unsafe { Vec::from_raw_parts(self.ptr, self.len, self.len) });
        }
    }
}

/// Convert an `Option<T>` into a nullable heap-allocated pointer.
///
/// - `Some(v)` is boxed and returned as a raw pointer
/// - `None` returns null
///
/// The returned pointer (if non-null) must eventually be freed with
/// [`free_ptr`] or the appropriate generated free function.
pub fn option_to_ptr<T>(v: Option<T>) -> *mut T {
    match v {
        Some(val) => Box::into_raw(Box::new(val)),
        None => ptr::null_mut(),
    }
}

/// Free a heap-allocated value previously created by [`option_to_ptr`]
/// or `Box::into_raw`.
///
/// If the pointer is null, this is a no-op.
///
/// # Safety
///
/// The pointer must have been allocated by `Box::into_raw` from this
/// process, or be null.
pub unsafe fn free_ptr<T>(ptr: *mut T) {
    if !ptr.is_null() {
        drop(unsafe { Box::from_raw(ptr) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_buffer_from_string() {
        let s = "hello world".to_string();
        let buf = FfiByteBuffer::from_string(s);
        assert!(!buf.ptr.is_null());
        assert_eq!(buf.len, 11);

        // Read back the bytes
        let bytes = unsafe { std::slice::from_raw_parts(buf.ptr, buf.len) };
        assert_eq!(bytes, b"hello world");

        // Free
        unsafe { buf.free() };
    }

    #[test]
    fn test_byte_buffer_from_vec() {
        let v = vec![1u8, 2, 3, 4];
        let buf = FfiByteBuffer::from_vec(v);
        assert_eq!(buf.len, 4);

        let bytes = unsafe { std::slice::from_raw_parts(buf.ptr, buf.len) };
        assert_eq!(bytes, &[1, 2, 3, 4]);

        unsafe { buf.free() };
    }

    #[test]
    fn test_byte_buffer_empty() {
        let buf = FfiByteBuffer::empty();
        assert!(buf.ptr.is_null());
        assert_eq!(buf.len, 0);

        // Freeing an empty buffer should be a no-op
        unsafe { buf.free() };
    }

    #[test]
    fn test_byte_slice_as_str() {
        let data = b"hello";
        let slice = FfiByteSlice {
            ptr: data.as_ptr(),
            len: data.len(),
        };

        let s = unsafe { slice.as_str_unchecked() };
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_option_to_ptr_some() {
        let ptr = option_to_ptr(Some(42u64));
        assert!(!ptr.is_null());
        assert_eq!(unsafe { *ptr }, 42);
        unsafe { free_ptr(ptr) };
    }

    #[test]
    fn test_option_to_ptr_none() {
        let ptr: *mut u64 = option_to_ptr(None);
        assert!(ptr.is_null());
        // Freeing null should be a no-op
        unsafe { free_ptr(ptr) };
    }
}
