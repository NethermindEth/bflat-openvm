/**
 * @file
 * @brief Keccak-256 for OpenVM, built on the native keccak-f[1600] instruction.
 *
 * OpenVM accelerates the permutation, not the sponge, so the padding and
 * absorb/squeeze loop live here. Two instructions do the work:
 *
 *   zkvm_keccak_xorin(state, input, len)  absorbs up to KECCAK_RATE bytes
 *   zkvm_keccakf(state)                   permutes the 200-byte state
 *
 * Both come from src/openvm_intrinsics and require 8-byte aligned pointers,
 * which is why the state is declared with an alignment attribute and the
 * trailing partial block is staged through an aligned scratch buffer.
 *
 * Copyright (C) 2026 Demerzel Solutions Limited (Nethermind)
 */
#include "zkvm_accelerators.h"

#define KECCAK_WIDTH 200
#define KECCAK_RATE  136   /* 1600 - 2*256 bits, the Keccak-256 rate */

extern void zkvm_keccakf(uint8_t *state);
extern void zkvm_keccak_xorin(uint8_t *state, const uint8_t *input, size_t len);

zkvm_status
zkvm_keccak256(const uint8_t *data, size_t len, zkvm_keccak256_hash *output)
{
    if (output == 0 || (data == 0 && len != 0))
        return ZKVM_EFAIL;

    uint8_t state[KECCAK_WIDTH] __attribute__((aligned(8)));
    uint8_t block[KECCAK_RATE] __attribute__((aligned(8)));

    for (size_t i = 0; i < KECCAK_WIDTH; i++)
        state[i] = 0;

    /* Absorb every whole rate-sized block. The instruction caps at
     * KECCAK_RATE bytes per call - a longer absorb executes but fails to
     * prove - so the loop never hands it more than one block. */
    while (len >= KECCAK_RATE) {
        zkvm_keccak_xorin(state, data, KECCAK_RATE);
        zkvm_keccakf(state);
        data += KECCAK_RATE;
        len -= KECCAK_RATE;
    }

    /* Final block: the remainder plus Keccak's 0x01 .. 0x80 padding. Staged
     * in an aligned local because `data + n` is very unlikely to be aligned
     * and the instruction requires it. */
    for (size_t i = 0; i < KECCAK_RATE; i++)
        block[i] = 0;
    for (size_t i = 0; i < len; i++)
        block[i] = data[i];
    block[len] |= 0x01;                 /* Keccak (not SHA-3) domain padding */
    block[KECCAK_RATE - 1] |= 0x80;

    zkvm_keccak_xorin(state, block, KECCAK_RATE);
    zkvm_keccakf(state);

    for (size_t i = 0; i < 32; i++)
        output->bytes[i] = state[i];

    return ZKVM_EOK;
}
