/**
 * @file
 * @brief eth-act zkVM accelerator interface, as implemented for OpenVM.
 *
 * Mirrors the prototypes from the zkVM Standards for Ethereum
 * (https://github.com/eth-act/zkvm-standards). Only the declarations this
 * library actually defines are listed; see README.md for coverage.
 *
 * Copyright (C) 2026 Demerzel Solutions Limited (Nethermind)
 */
#ifndef BFLAT_OPENVM_ZKVM_ACCELERATORS_H
#define BFLAT_OPENVM_ZKVM_ACCELERATORS_H

#include <stddef.h>
#include <stdint.h>

typedef enum {
    ZKVM_EOK = 0,
    ZKVM_EFAIL = -1
} zkvm_status;

typedef struct { uint8_t bytes[32]; } zkvm_bytes_32;

typedef zkvm_bytes_32 zkvm_keccak256_hash;
typedef zkvm_bytes_32 zkvm_sha256_hash;

/** Keccak-256 over @p len bytes at @p data. */
zkvm_status zkvm_keccak256(const uint8_t *data, size_t len, zkvm_keccak256_hash *output);

/** SHA-256 over @p len bytes at @p data. */
zkvm_status zkvm_sha256(const uint8_t *data, size_t len, zkvm_sha256_hash *output);

#endif /* BFLAT_OPENVM_ZKVM_ACCELERATORS_H */
