//! `zkvm_keccak256` and `zkvm_sha256`, backed by OpenVM's keccak256 and sha2
//! extensions. Both delegate to OpenVM's own guest libraries rather than
//! reimplementing the sponge or the message schedule here.

use openvm_sha2::{Digest, Sha256};

use crate::types::{ZkvmKeccak256Hash, ZkvmSha256Hash, ZkvmStatus};

/// Computes the Keccak-256 hash of `data[..len]` into `output`.
///
/// # Safety
/// - `data` must be valid for reads of `len` bytes (ignored when `len == 0`).
/// - `output` must be valid for writes of 32 bytes.
#[no_mangle]
pub unsafe extern "C" fn zkvm_keccak256(
    data: *const u8,
    len: usize,
    output: *mut ZkvmKeccak256Hash,
) -> ZkvmStatus {
    if output.is_null() || (len > 0 && data.is_null()) {
        return ZkvmStatus::Fail;
    }
    unsafe {
        openvm_keccak256::native_keccak256(data, len, (*output).data.as_mut_ptr());
    }
    ZkvmStatus::Ok
}

/// Computes the SHA-256 hash of `data[..len]` into `output`.
///
/// # Safety
/// - `data` must be valid for reads of `len` bytes (ignored when `len == 0`).
/// - `output` must be valid for writes of 32 bytes.
#[no_mangle]
pub unsafe extern "C" fn zkvm_sha256(
    data: *const u8,
    len: usize,
    output: *mut ZkvmSha256Hash,
) -> ZkvmStatus {
    if output.is_null() || (len > 0 && data.is_null()) {
        return ZkvmStatus::Fail;
    }
    unsafe {
        let input = if len == 0 {
            &[][..]
        } else {
            core::slice::from_raw_parts(data, len)
        };
        (*output).data = Sha256::digest(input).into();
    }
    ZkvmStatus::Ok
}
