# bflat-openvm

[![Nethermind.OpenVm.Runtime](https://img.shields.io/nuget/v/Nethermind.OpenVm.Runtime)](https://www.nuget.org/packages/Nethermind.OpenVm.Runtime)

OpenVM bindings for [bflat-riscv64](https://github.com/NethermindEth/bflat-riscv64)
guests built with `--libc openvm`, providing the native half of
[Nethermind.Zkvm.Abstractions](https://www.nuget.org/packages/Nethermind.Zkvm.Abstractions).

## What this is

`libopenvm.a` — two layers in one archive:

1. **`src/accel-rs`** — a Rust staticlib implementing the
   [eth-act zkVM Cryptographic Accelerators C Interface](https://github.com/eth-act/zkvm-standards/tree/main/standards/c-interface-accelerators)
   on top of OpenVM's own accelerated guest libraries (`openvm-keccak256`,
   `openvm-sha2`, and the accelerated `k256` / `p256` forks).
2. **`src/openvm_intrinsics`** — one instruction per accelerated primitive, for
   anything the standard does not cover or that managed code wants to reach
   directly. OpenVM has no syscalls: each primitive is a custom instruction
   under RISC-V custom-0 (`0x0b`) or custom-1 (`0x2b`).

Dependencies are pinned to a commit on `develop-v2.1.0`, OpenVM's live RV64
line — not `develop-v2.1.0-rv64` (an older snapshot of the same work) and not
`main` (still RV32, and it rejects a 64-bit ELF).

## Usage

```console
$ bflat build app.cs --os linux --arch riscv64 --libc openvm \
      --extlib path/to/libopenvm.bflat.manifest
```

## Building and testing

```console
$ ./build/build.sh          # needs binutils-riscv64-linux-gnu + clang
$ ./tests/run_tests.sh      # host tests, no zkVM needed
```

## Coverage

| Standard entry point | State |
|----------------------|-------|
| `zkvm_keccak256` | **implemented** (`openvm-keccak256`) |
| `zkvm_sha256` | **implemented** (`openvm-sha2`) |
| `zkvm_secp256k1_verify` | **implemented** (accelerated `k256`) |
| `zkvm_secp256k1_ecrecover` | **implemented** (accelerated `k256`) |
| `zkvm_secp256r1_verify` | **implemented** (accelerated `p256`) |
| `ripemd160`, `blake2f`, `modexp`, BN254, BLS12-381, KZG | not implemented |

OpenVM has an official implementation of the same interface in progress — the
`openvm-caci` crate on the unmerged branch `feat/caci-secp256r1`, covering the
same five functions. It has no pull request and has not moved since 2026-07-28,
so this package implements them itself rather than pinning to a stalled branch.

| Standard entry point | State |
|----------------------|-------|
| `zkvm_keccak256` | **implemented**, on the native keccak-f[1600] + xorin instructions |
| `zkvm_sha256` | **implemented**, on the native SHA-256 compression instruction |
| everything else | not implemented |

The 19-function standard also covers `ripemd160`, `blake2f`, `modexp`,
secp256k1/secp256r1, BN254, BLS12-381 and KZG. Three of those
(secp256k1 verify/ecrecover, secp256r1 verify) already exist upstream in
`openvm-caci`; the rest do not. OpenVM's curve support (`openvm-pairing`,
`openvm-k256`, `openvm-p256`, and `openvm-kzg` from `axiom-crypto/openvm-eth`)
is exposed as generic Rust APIs over traits, with no `no_mangle` entry point to
bind to, so closing the remaining gap means either wrapping them by hand — one
monomorphised instance per curve, in a Rust staticlib alongside this one — or
computing them in managed code.

## Why the curves need an explicit init

OpenVM's algebra and ecc extensions split each accelerated operation in two:
the library (`k256`, `p256`) *declares* it as an `extern "C"` symbol, and the
program *defines* it by naming the concrete moduli and curves it uses — what
`cargo openvm` generates as `openvm_init_*.rs` for a Rust guest. Our guest is
C#, and this staticlib is the only Rust in the link, so that declaration lives
in `src/accel-rs/src/init.rs`.

Without it everything still compiles, but every `sw_add_ne_extern_func_*` and
`moduli_setup_extern_func_*` stays undefined and not one custom-1 instruction
is emitted — the curve work would quietly fall back to portable Rust. `build.sh`
therefore counts both custom-0 and custom-1 instructions in the finished archive
and fails if either is zero.

## Constraints

**Every pointer passed to the intrinsics must be 8-byte aligned.** OpenVM's own
Rust wrappers spend most of their body bouncing unaligned buffers through a
temporary; these shims do not, which is what keeps each primitive at one
instruction. The accelerator entry points above stage caller buffers through
aligned locals, so this constraint applies only if you call the intrinsics
directly.

**`zkvm_keccak_xorin` absorbs at most 136 bytes** (`KECCAK_RATE`); a larger
length executes but fails to prove. `zkvm_keccak256` chunks for you.

## Verification

The hashes and curves are OpenVM's own implementations, so there is no local
crypto to test. What `build.sh` does check, on every build, is that the
accelerators actually activated: it counts custom-0 and custom-1 instructions
in the archive and fails on zero. CI additionally asserts that every symbol the
managed side imports is present and global, and that none of OpenVM's runtime
entry points leaked in as a global.
