//! Curve and modulus registration.
//!
//! OpenVM's algebra and ecc extensions split their accelerated operations in
//! two: the library (`k256`, `p256`) DECLARES each operation as an
//! `extern "C"` symbol, and the program DEFINES it by naming the concrete
//! moduli and curves it uses. `cargo openvm` normally generates that
//! declaration into an `openvm_init_*.rs` for a Rust guest.
//!
//! Our guest is C#, and this staticlib is the only Rust in the link, so the
//! declaration lives here. Without it the accelerated paths compile but every
//! `sw_add_ne_extern_func_*` / `moduli_setup_extern_func_*` stays undefined,
//! and no custom-1 (0x2b) instruction is ever emitted - the curve work would
//! silently fall back to portable Rust, which is exactly what these
//! accelerators exist to avoid.
//!
//! The four moduli are, in order, the secp256k1 field and group orders and the
//! secp256r1 field and group orders. They must be listed in the same order the
//! declaring crates expect.

#[allow(unused_imports)]
use openvm_k256::Secp256k1Point;
#[allow(unused_imports)]
use openvm_p256::P256Point;

openvm_algebra_guest::moduli_macros::moduli_init! {
    // secp256k1: field modulus p, then group order n
    "115792089237316195423570985008687907853269984665640564039457584007908834671663",
    "115792089237316195423570985008687907852837564279074904382605163141518161494337",
    // secp256r1: field modulus p, then group order n
    "115792089210356248762697446949407573530086143415290314195533631308867097853951",
    "115792089210356248762697446949407573529996955224135760342422259061068512044369"
}

openvm_ecc_guest::sw_macros::sw_init! {
    "Secp256k1Point",
    "P256Point"
}
