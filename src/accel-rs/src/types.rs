//! Rust mirrors of the C types in the standard's `zkvm_accelerators.h`.

/// `zkvm_status`. C declares it as an enum with a negative member, which the
/// C ABI represents as `int`.
#[repr(i32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ZkvmStatus {
    /// `ZKVM_EOK`
    Ok = 0,
    /// `ZKVM_EFAIL`
    Fail = -1,
}

macro_rules! zkvm_bytes {
    ($name:ident, $len:expr) => {
        #[repr(C, align(8))]
        pub struct $name {
            pub data: [u8; $len],
        }
    };
}

zkvm_bytes!(ZkvmBytes32, 32);
zkvm_bytes!(ZkvmBytes64, 64);

pub type ZkvmKeccak256Hash = ZkvmBytes32;
pub type ZkvmSha256Hash = ZkvmBytes32;
pub type ZkvmSecp256k1Hash = ZkvmBytes32;
pub type ZkvmSecp256k1Signature = ZkvmBytes64;
pub type ZkvmSecp256k1Pubkey = ZkvmBytes64;
pub type ZkvmSecp256r1Hash = ZkvmBytes32;
pub type ZkvmSecp256r1Signature = ZkvmBytes64;
pub type ZkvmSecp256r1Pubkey = ZkvmBytes64;
