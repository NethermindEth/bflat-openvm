/**
 * @file
 * @brief OpenVM intrinsics (custom-instruction based)
 *
 * OpenVM has no syscalls at all: every accelerated primitive is a custom
 * instruction under RISC-V custom-0 (opcode 0x0b), distinguished by funct3 and
 * funct7. Encodings are taken from the guest crates on the RV64 branch
 * (openvm, branch develop-v2.1.0-rv64):
 *
 *   extensions/keccak256/guest/src/lib.rs   funct3 0b100, funct7 0 (keccakf), 1 (xorin)
 *   extensions/sha2/guest/src/lib.rs        funct3 0b100, funct7 2 (sha256), 3 (sha512)
 *   extensions/bigint/guest/src/lib.rs      funct3 0b101, funct7 = Int256Funct7
 *
 * ALIGNMENT IS THE CALLER'S JOB. OpenVM's own Rust wrappers spend most of
 * their body bouncing unaligned buffers through a temporary; these shims do
 * not, so every pointer passed here MUST be 8-byte aligned. That is free for
 * us - the guest heap hands out 8-aligned blocks - and it keeps each
 * primitive at exactly one instruction.
 *
 * Copyright (C) 2026 Demerzel Solutions Limited (Nethermind)
 *
 * @author Maxim Menshikov <maksim.menshikov@nethermind.io>
 */
.option norelax

/* R-type custom instruction over three pointer arguments:
 *   rd = a0, rs1 = a1, rs2 = a2 - matching the (result, a, b) order the
 *   OpenVM externs use. */
.macro OPENVM_R3 name, funct3, funct7
    .section .text, "ax", @progbits
    .balign 4
    .global \name
    .type \name, @function
\name:
    .cfi_startproc
    /* leaf: no stack frame */
    .insn   r 0x0b, \funct3, \funct7, a0, a1, a2
    ret
    .cfi_endproc
    .size \name, .-\name
.endm

/* R-type over a single in/out buffer: rd = a0, rs1 = rs2 = x0. */
.macro OPENVM_R1 name, funct3, funct7
    .section .text, "ax", @progbits
    .balign 4
    .global \name
    .type \name, @function
\name:
    .cfi_startproc
    /* leaf: no stack frame */
    .insn   r 0x0b, \funct3, \funct7, a0, x0, x0
    ret
    .cfi_endproc
    .size \name, .-\name
.endm

/* ---- Keccak-f[1600] ----------------------------------------------------
 * keccakf(buffer): permutes 200 bytes in place.
 * xorin(buffer, input, len): absorbs len bytes; len <= 136 (KECCAK_RATE) -
 * a larger value executes but fails to prove, so the managed side must
 * chunk. */
OPENVM_R1  zkvm_keccakf,                0b100, 0
OPENVM_R3  zkvm_keccak_xorin,           0b100, 1

/* ---- SHA-2 compression -------------------------------------------------
 * zkvm_sha256_impl(state, input, output) - note the register order: the
 * instruction takes rd = output, rs1 = state, rs2 = input, so the shim
 * shuffles the C arguments into place rather than using OPENVM_R3. */
    .section .text, "ax", @progbits
    .balign 4
    .global zkvm_sha256_compress
    .type zkvm_sha256_compress, @function
zkvm_sha256_compress:
    .cfi_startproc
    .insn   r 0x0b, 0b100, 2, a2, a0, a1
    ret
    .cfi_endproc
    .size zkvm_sha256_compress, .-zkvm_sha256_compress

    .section .text, "ax", @progbits
    .balign 4
    .global zkvm_sha512_compress
    .type zkvm_sha512_compress, @function
zkvm_sha512_compress:
    .cfi_startproc
    .insn   r 0x0b, 0b100, 3, a2, a0, a1
    ret
    .cfi_endproc
    .size zkvm_sha512_compress, .-zkvm_sha512_compress

/* ---- 256-bit integer arithmetic ----------------------------------------
 * All take (result, a, b) over 32-byte little-endian operands. funct7 is the
 * Int256Funct7 discriminant. */
OPENVM_R3  zkvm_u256_add,               0b101, 0
OPENVM_R3  zkvm_u256_sub,               0b101, 1
OPENVM_R3  zkvm_u256_xor,               0b101, 2
OPENVM_R3  zkvm_u256_or,                0b101, 3
OPENVM_R3  zkvm_u256_and,               0b101, 4
OPENVM_R3  zkvm_u256_sll,               0b101, 5
OPENVM_R3  zkvm_u256_srl,               0b101, 6
OPENVM_R3  zkvm_u256_sra,               0b101, 7
OPENVM_R3  zkvm_u256_slt,               0b101, 8
OPENVM_R3  zkvm_u256_sltu,              0b101, 9
OPENVM_R3  zkvm_u256_mul,               0b101, 10
