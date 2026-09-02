//! Curve, modulus and field-extension registration.
//!
//! OpenVM's algebra, ecc and pairing extensions split their accelerated
//! operations in two: the library (`k256`, `p256`, `openvm-pairing`) DECLARES
//! each operation as an `extern "C"` symbol, and the program DEFINES it by
//! naming the concrete moduli, complex extensions and curves it uses.
//! `cargo openvm` normally generates that into an `openvm_init_*.rs` for a
//! Rust guest.
//!
//! Our guest is C#, and this staticlib is the only Rust in the link, so the
//! declaration lives here. Without it the accelerated paths still compile, but
//! every `sw_add_ne_extern_func_*`, `moduli_setup_extern_func_*` and
//! `complex_*_extern_func_*` stays undefined and not one custom-1 (0x2b)
//! instruction is emitted - the curve work would silently fall back to
//! portable Rust, which is exactly what these accelerators exist to avoid.
//! `build/build.sh` counts the instructions and fails the build on zero.
//!
//! THE ORDER OF THE MODULI IS PART OF THE ABI. `complex_init!` addresses its
//! base field by index into this list, so bn254's Fp must stay at 0 and
//! bls12-381's Fp at 6. This is the same list, in the same order, that
//! ere-guests generates for its OpenVM Ethereum guest
//! (`bin/stateless-validator-reth/openvm/openvm_init.rs`) - kept identical on
//! purpose, so the two can be compared symbol for symbol.

#[allow(unused_imports)]
use openvm_k256::Secp256k1Point;
#[allow(unused_imports)]
use openvm_p256::P256Point;
#[allow(unused_imports)]
use openvm_pairing::{
    bls12_381::{Bls12_381Fp2, G1Affine as Bls12_381G1Affine},
    bn254::{Bn254Fp2, G1Affine as Bn254G1Affine},
};

openvm_algebra_guest::moduli_macros::moduli_init! {
    // 0: BN254 field modulus p
    "21888242871839275222246405745257275088696311157297823662689037894645226208583",
    // 1: BN254 group order r
    "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    // 2: secp256k1 field modulus p
    "115792089237316195423570985008687907853269984665640564039457584007908834671663",
    // 3: secp256k1 group order n
    "115792089237316195423570985008687907852837564279074904382605163141518161494337",
    // 4: secp256r1 field modulus p
    "115792089210356248762697446949407573530086143415290314195533631308867097853951",
    // 5: secp256r1 group order n
    "115792089210356248762697446949407573529996955224135760342422259061068512044369",
    // 6: BLS12-381 field modulus p
    "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559787",
    // 7: BLS12-381 group order r
    "52435875175126190479447740508185965837690552500527637822603658699938581184513"
}

openvm_algebra_guest::complex_macros::complex_init! {
    "Bn254Fp2" { mod_idx = 0 },
    "Bls12_381Fp2" { mod_idx = 6 }
}

openvm_ecc_guest::sw_macros::sw_init! {
    "Bn254G1Affine",
    "Secp256k1Point",
    "P256Point",
    "Bls12_381G1Affine"
}
