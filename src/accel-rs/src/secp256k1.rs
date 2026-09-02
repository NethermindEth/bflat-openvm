//! `zkvm_secp256k1_verify` and `zkvm_secp256k1_ecrecover`, backed by OpenVM's
//! ecc extension through the accelerated `k256` fork.

use openvm_k256::ecdsa::{signature::hazmat::PrehashVerifier, RecoveryId, Signature, VerifyingKey};

use crate::types::{ZkvmSecp256k1Hash, ZkvmSecp256k1Pubkey, ZkvmSecp256k1Signature, ZkvmStatus};

/// Verifies an ECDSA/secp256k1 signature over a prehashed message.
///
/// The standard passes the public key as the bare 64-byte `x || y`, while
/// `k256` wants SEC1, so the uncompressed tag is prepended here.
///
/// # Safety
/// - `msg`, `sig` and `pubkey` must be valid for reads of 32, 64 and 64 bytes.
/// - `verified` must be valid for writes of 1 byte.
#[no_mangle]
pub unsafe extern "C" fn zkvm_secp256k1_verify(
    msg: *const ZkvmSecp256k1Hash,
    sig: *const ZkvmSecp256k1Signature,
    pubkey: *const ZkvmSecp256k1Pubkey,
    verified: *mut bool,
) -> ZkvmStatus {
    if msg.is_null() || sig.is_null() || pubkey.is_null() || verified.is_null() {
        return ZkvmStatus::Fail;
    }
    unsafe {
        let mut sec1 = [0u8; 65];
        sec1[0] = 0x04;
        sec1[1..].copy_from_slice(&core::ptr::read(pubkey).data);
        let msg_bytes = core::ptr::read(msg).data;
        let sig_bytes = core::ptr::read(sig).data;

        let Ok(vk) = VerifyingKey::from_sec1_bytes(&sec1) else {
            return ZkvmStatus::Fail;
        };
        let Ok(signature) = Signature::try_from(&sig_bytes[..]) else {
            return ZkvmStatus::Fail;
        };
        *verified = vk.verify_prehash(&msg_bytes, &signature).is_ok();
    }
    ZkvmStatus::Ok
}

/// Recovers the public key that signed a prehashed message - the ecrecover
/// precompile. Writes the uncompressed key as `x || y`, without the SEC1 tag.
///
/// # Safety
/// - `msg` and `sig` must be valid for reads of 32 and 64 bytes.
/// - `output` must be valid for writes of 64 bytes.
#[no_mangle]
pub unsafe extern "C" fn zkvm_secp256k1_ecrecover(
    msg: *const ZkvmSecp256k1Hash,
    sig: *const ZkvmSecp256k1Signature,
    recid: u8,
    output: *mut ZkvmSecp256k1Pubkey,
) -> ZkvmStatus {
    if msg.is_null() || sig.is_null() || output.is_null() {
        return ZkvmStatus::Fail;
    }
    let Some(recovery_id) = RecoveryId::from_byte(recid) else {
        return ZkvmStatus::Fail;
    };
    unsafe {
        let msg_bytes = core::ptr::read(msg).data;
        let sig_bytes = core::ptr::read(sig).data;
        let Ok(signature) = Signature::try_from(&sig_bytes[..]) else {
            return ZkvmStatus::Fail;
        };
        let Ok(vk) = VerifyingKey::recover_from_prehash(&msg_bytes, &signature, recovery_id)
        else {
            return ZkvmStatus::Fail;
        };
        // to_encoded_point(false) is SEC1 uncompressed: 0x04 || x || y. The
        // standard wants x || y, so the tag is dropped.
        let point = vk.to_encoded_point(false);
        let bytes = point.as_bytes();
        if bytes.len() != 65 {
            return ZkvmStatus::Fail;
        }
        (*output).data.copy_from_slice(&bytes[1..]);
    }
    ZkvmStatus::Ok
}
