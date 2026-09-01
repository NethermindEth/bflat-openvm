#!/bin/bash
# Builds libopenvm.a - the OpenVM precompile shim linked into bflat guests via
# --extlib. Unlike bflat-libziskos there is no upstream library to build:
# every OpenVM primitive is one instruction, so the whole package is the
# hand-written assembly in src/openvm_intrinsics.
#
# Reference for the encodings: openvm (develop-v2.1.0-rv64) extensions/*/guest
set -e

fail() { echo "$@" >&2; exit 1; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/output"

AS="${AS:-riscv64-linux-gnu-as}"
AR="${AR:-riscv64-linux-gnu-ar}"
RANLIB="${RANLIB:-riscv64-linux-gnu-ranlib}"

command -v "${AS}" >/dev/null 2>&1 || fail "${AS} not found (apt install binutils-riscv64-linux-gnu)"

mkdir -p "${OUTPUT_DIR}"
rm -f "${OUTPUT_DIR}/libopenvm.a"

# -march=rv64im -mabi=lp64: OpenVM decodes the base integer set with M only, and
# the guest is linked soft-float. The ABI marker is normalised to soft-float
# below so ld.lld accepts these objects against the guest's crt1.o.
echo "Assembling openvm_intrinsics.S..."
"${AS}" --march=rv64im --mabi=lp64 \
    "${ROOT_DIR}/src/openvm_intrinsics/openvm_intrinsics.S" \
    -o "${OUTPUT_DIR}/openvm_intrinsics.o" || fail "assembly failed"

# Clear the ELF flags byte: binutils marks the object with the float ABI it
# was assembled for, and a mismatch makes ld.lld reject every member.
printf '\x00' | dd of="${OUTPUT_DIR}/openvm_intrinsics.o" bs=1 seek=48 count=1 conv=notrunc status=none

# The eth-act accelerator entry points. OpenVM accelerates the Keccak
# permutation and the SHA-2 compression function but not the hashes
# themselves, so the sponge, the padding and the block loops are compiled
# here on top of the intrinsics above. Everything else the standard defines
# is not implemented - see README.md.
CC="${CC:-clang}"
command -v "${CC}" >/dev/null 2>&1 || fail "${CC} not found"
ACCEL_OBJS=""
for src in keccak256 sha256 ; do
    echo "Compiling ${src}.c..."
    "${CC}" --target=riscv64-unknown-elf -march=rv64im -mabi=lp64 -mcmodel=medany \
        -ffreestanding -fno-builtin -fno-stack-protector -nostdlibinc \
        -O2 -std=gnu2x -Wall \
        -c "${ROOT_DIR}/src/accelerators/${src}.c" \
        -o "${OUTPUT_DIR}/${src}.o" || fail "failed to compile ${src}.c"
    printf '\x00' | dd of="${OUTPUT_DIR}/${src}.o" bs=1 seek=48 count=1 conv=notrunc status=none
    ACCEL_OBJS="${ACCEL_OBJS} ${OUTPUT_DIR}/${src}.o"
done

echo "Creating libopenvm.a..."
"${AR}" rcs "${OUTPUT_DIR}/libopenvm.a" "${OUTPUT_DIR}/openvm_intrinsics.o" ${ACCEL_OBJS} || fail "ar failed"
"${RANLIB}" "${OUTPUT_DIR}/libopenvm.a" || fail "ranlib failed"

cp "${ROOT_DIR}/bflat-manifest.json" "${OUTPUT_DIR}/libopenvm.bflat.manifest"

echo "Build completed"
echo "Output: ${OUTPUT_DIR}/libopenvm.a ($("${AR}" t "${OUTPUT_DIR}/libopenvm.a" | wc -l) member(s))"
echo "        ${OUTPUT_DIR}/libopenvm.bflat.manifest"
