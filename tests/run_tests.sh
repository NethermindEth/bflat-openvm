#!/bin/bash
# Host tests for the accelerator glue. Runs natively - no zkVM, no emulator:
# the OpenVM custom instructions are replaced by software stand-ins so the
# sponge/padding/block-loop code can be checked against known vectors.
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
CC="${CC:-clang}"
OUT="$(mktemp -d)"
trap 'rm -rf "${OUT}"' EXIT

"${CC}" -O1 -Wall -std=gnu2x \
    "${SCRIPT_DIR}/test_accelerators.c" \
    "${ROOT_DIR}/src/accelerators/keccak256.c" \
    "${ROOT_DIR}/src/accelerators/sha256.c" \
    -o "${OUT}/test_accelerators"

"${OUT}/test_accelerators"
