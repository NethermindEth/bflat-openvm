/**
 * @file
 * @brief SHA-256 for OpenVM, built on the native compression instruction.
 *
 * OpenVM accelerates the compression function, not the whole hash, so the
 * message schedule padding and the block loop live here:
 *
 *   zkvm_sha256_compress(state, input, output)
 *
 * takes a 32-byte state and a 64-byte block and writes the next state as
 * 8 big-endian 32-bit words. All three pointers must be 8-byte aligned
 * (src/openvm_intrinsics), hence the aligned locals.
 *
 * Copyright (C) 2026 Demerzel Solutions Limited (Nethermind)
 */
#include "zkvm_accelerators.h"

#define SHA256_BLOCK 64

extern void zkvm_sha256_compress(const uint8_t *state, const uint8_t *input, uint8_t *output);

/* SHA-256 IV, big-endian bytes: the standard's H(0) constants. */
static const uint8_t SHA256_IV[32] = {
    0x6a,0x09,0xe6,0x67, 0xbb,0x67,0xae,0x85, 0x3c,0x6e,0xf3,0x72, 0xa5,0x4f,0xf5,0x3a,
    0x51,0x0e,0x52,0x7f, 0x9b,0x05,0x68,0x8c, 0x1f,0x83,0xd9,0xab, 0x5b,0xe0,0xcd,0x19
};

zkvm_status
zkvm_sha256(const uint8_t *data, size_t len, zkvm_sha256_hash *output)
{
    if (output == 0 || (data == 0 && len != 0))
        return ZKVM_EFAIL;

    uint8_t state[32] __attribute__((aligned(8)));
    uint8_t next[32] __attribute__((aligned(8)));
    uint8_t block[SHA256_BLOCK] __attribute__((aligned(8)));

    for (size_t i = 0; i < 32; i++)
        state[i] = SHA256_IV[i];

    const uint64_t bit_len = (uint64_t)len * 8u;

    while (len >= SHA256_BLOCK) {
        /* Stage through an aligned local: the caller's buffer carries no
         * alignment guarantee and the instruction demands one. */
        for (size_t i = 0; i < SHA256_BLOCK; i++)
            block[i] = data[i];
        zkvm_sha256_compress(state, block, next);
        for (size_t i = 0; i < 32; i++)
            state[i] = next[i];
        data += SHA256_BLOCK;
        len -= SHA256_BLOCK;
    }

    /* Padding: 0x80, zeroes, then the 64-bit big-endian bit length. If the
     * remainder leaves no room for the length field, emit an extra block. */
    for (size_t i = 0; i < SHA256_BLOCK; i++)
        block[i] = 0;
    for (size_t i = 0; i < len; i++)
        block[i] = data[i];
    block[len] = 0x80;

    if (len >= SHA256_BLOCK - 8) {
        zkvm_sha256_compress(state, block, next);
        for (size_t i = 0; i < 32; i++)
            state[i] = next[i];
        for (size_t i = 0; i < SHA256_BLOCK; i++)
            block[i] = 0;
    }

    for (size_t i = 0; i < 8; i++)
        block[SHA256_BLOCK - 1 - i] = (uint8_t)(bit_len >> (8 * i));

    zkvm_sha256_compress(state, block, next);

    for (size_t i = 0; i < 32; i++)
        output->bytes[i] = next[i];

    return ZKVM_EOK;
}
