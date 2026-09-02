//! OpenVM implementation of the
//! [eth-act zkVM Cryptographic Accelerators C Interface](https://github.com/eth-act/zkvm-standards/tree/main/standards/c-interface-accelerators).
//!
//! The guest is C#/NativeAOT, so it cannot reach OpenVM's accelerators the way
//! Rust guests do - by swapping crypto crates through `[patch.crates-io]`. It
//! goes through this C ABI instead: each function unpacks the standard's byte
//! buffers, calls an OpenVM-accelerated guest library, and packs the result
//! back.
//!
//! MUST be built for a target whose `os` is `openvm` (see
//! riscv64im-openvm-elf.json), or with `--cfg openvm_intrinsics`. OpenVM gates
//! every accelerated path behind `cfg(any(openvm_intrinsics, target_os =
//! "openvm"))`; on any other target this still compiles and links, but each
//! primitive silently falls back to portable Rust and the point is lost.
#![no_std]

extern crate alloc;

mod hash;
mod init;
mod secp256k1;
mod secp256r1;
mod types;

// No `#[global_allocator]` and no `#[panic_handler]` here: pulling in the
// accelerated `k256` brings the whole `openvm` guest runtime, which already
// provides both, and a second definition of either is a hard error.
//
// That runtime's allocator is a bump pointer over `sys_alloc_aligned`, which
// bflat's `rust_sys` module wraps (`--wrap=sys_alloc_aligned`) and routes into
// pal's heap - so the Rust side and the C# guest share one heap instead of the
// staticlib carving out a second, invisible one.
