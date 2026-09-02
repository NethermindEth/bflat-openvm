//! `zkvm_secp256r1_verify`, backed by OpenVM's ecc extension through the
//! accelerated `p256` fork.

use openvm_p256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};

use crate::types::{ZkvmSecp256r1Hash, ZkvmSecp256r1Pubkey, ZkvmSecp256r1Signature, ZkvmStatus};

/// Verifies an ECDSA/secp256r1 signature over a prehashed message.
///
/// # Safety
/// - `msg`, `sig` and `pubkey` must be valid for reads of 32, 64 and 64 bytes.
/// - `verified` must be valid for writes of 1 byte.
#[no_mangle]
pub unsafe extern "C" fn zkvm_secp256r1_verify(
    msg: *const ZkvmSecp256r1Hash,
    sig: *const ZkvmSecp256r1Signature,
    pubkey: *const ZkvmSecp256r1Pubkey,
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
