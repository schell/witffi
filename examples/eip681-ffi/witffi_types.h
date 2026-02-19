/* witffi_types.h — Shared FFI types for witffi-generated code. */
/* This header is part of the witffi-types crate. Do not edit. */
#pragma once

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* A borrowed byte slice (caller-owned, const pointer). */
typedef struct {
    const uint8_t *ptr;
    size_t len;
} FfiByteSlice;

/* An owned byte buffer (callee-allocated, must be freed). */
typedef struct {
    uint8_t *ptr;
    size_t len;
} FfiByteBuffer;

#ifdef __cplusplus
}
#endif
