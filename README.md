# bflat-openvm

[![Nethermind.OpenVm.Runtime](https://img.shields.io/nuget/v/Nethermind.OpenVm.Runtime)](https://www.nuget.org/packages/Nethermind.OpenVm.Runtime)

OpenVM bindings for [bflat-riscv64](https://github.com/NethermindEth/bflat-riscv64)
guests built with `--libc openvm`, providing the native half of
[Nethermind.Zkvm.Abstractions](https://www.nuget.org/packages/Nethermind.Zkvm.Abstractions).

## What this is

`libopenvm.a` — two layers in one archive:

1. **Intrinsic shims** (`src/openvm_intrinsics`) — one instruction per
   accelerated primitive. OpenVM has no syscalls: each primitive is a custom
   instruction under RISC-V custom-0 (opcode `0x0b`).
2. **Accelerator entry points** (`src/accelerators`) — the
   [eth-act standard](https://github.com/eth-act/zkvm-standards) functions,
   written on top of those shims.

Encodings come from OpenVM's live RV64 line — branch `develop-v2.1.0`,
`extensions/{keccak256,sha2,bigint}/guest`. Note that `develop-v2.1.0-rv64` is
an older snapshot of the same work and `main` is still RV32, so neither is the
right reference. The encodings are identical across both RV64 snapshots; what
moved is the memory map, where guest memory grew from 512 MiB to the full 4 GiB
u32 range (`MEM_BITS` 29 -> 32).

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

OpenVM has an official implementation of the standard in progress — the
`openvm-caci` crate (`guest-libs/caci`), on the unmerged branch
`feat/caci-secp256r1`. It is RV64-based and currently covers `zkvm_keccak256`,
`zkvm_sha256`, `zkvm_secp256k1_verify`, `zkvm_secp256k1_ecrecover` and
`zkvm_secp256r1_verify`. It is an rlib, so consuming it would need a staticlib
facade (the shape SP1 uses for `libzkevm-cabi`) and a pinned dependency on a
feature branch.

Until that lands on a release branch this package implements the hashes
itself, directly on the custom instructions.

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

## Constraints

**Every pointer passed to the intrinsics must be 8-byte aligned.** OpenVM's own
Rust wrappers spend most of their body bouncing unaligned buffers through a
temporary; these shims do not, which is what keeps each primitive at one
instruction. The accelerator entry points above stage caller buffers through
aligned locals, so this constraint applies only if you call the intrinsics
directly.

**`zkvm_keccak_xorin` absorbs at most 136 bytes** (`KECCAK_RATE`); a larger
length executes but fails to prove. `zkvm_keccak256` chunks for you.

## Tests

`tests/test_accelerators.c` links the real sponge, padding and block-loop code
against *software* stand-ins for the two primitives OpenVM accelerates, then
checks NIST/Ethereum vectors — including the three cases where padding bugs
hide: rate−1 (padding start and `0x80` collide), an exact block (forces a whole
extra padding block), and multi-block input. The permutation and compression
function are OpenVM's to get right; the glue around them is ours.
