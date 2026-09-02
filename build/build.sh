#!/bin/bash
# Builds libopenvm.a - the native half of the OpenVM bindings for bflat guests.
#
# Two layers end up in one archive:
#
#   1. src/accel-rs  - a Rust staticlib implementing the eth-act zkVM
#      accelerator C ABI on top of OpenVM's accelerated guest libraries.
#   2. src/openvm_intrinsics - one instruction per accelerated primitive, for
#      anything the standard does not cover or that managed code wants to reach
#      directly.
#
# WHY A CUSTOM TARGET. OpenVM gates every accelerated path behind
# cfg(any(openvm_intrinsics, target_os = "openvm")). Built for a plain
# riscv64 target the crate still compiles and links, but each primitive
# silently falls back to portable Rust. riscv64im-openvm-elf.json sets
# os = "openvm", which flips those cfgs without needing OpenVM's forked
# toolchain. build.sh checks afterwards that the accelerated instructions are
# actually in the archive, so a silent fallback fails the build.
set -e

fail() { echo "$@" >&2; exit 1; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUTPUT_DIR="${ROOT_DIR}/output"
ACCEL_DIR="${ROOT_DIR}/src/accel-rs"
TARGET_JSON="${ACCEL_DIR}/riscv64im-openvm-elf.json"
TARGET_NAME="riscv64im-openvm-elf"

AS="${AS:-riscv64-linux-gnu-as}"
AR="${AR:-riscv64-linux-gnu-ar}"
RANLIB="${RANLIB:-riscv64-linux-gnu-ranlib}"
OBJCOPY="${OBJCOPY:-riscv64-linux-gnu-objcopy}"
CARGO="${CARGO:-cargo}"
# build-std needs nightly plus the rust-src component.
TOOLCHAIN="${TOOLCHAIN:-nightly}"

for tool in "${AS}" "${AR}" "${RANLIB}" "${OBJCOPY}" ; do
    command -v "${tool}" >/dev/null 2>&1 || fail "${tool} not found (apt install binutils-riscv64-linux-gnu)"
done
command -v rustup >/dev/null 2>&1 || command -v "${CARGO}" >/dev/null 2>&1 \
    || fail "cargo not found"

mkdir -p "${OUTPUT_DIR}"
rm -f "${OUTPUT_DIR}/libopenvm.a"

# --- 1. Rust accelerators --------------------------------------------------
echo "Building the accelerator staticlib for ${TARGET_NAME}..."
( cd "${ACCEL_DIR}" && rustup run "${TOOLCHAIN}" "${CARGO}" build --release \
    --target "${TARGET_JSON}" \
    -Z build-std=core,alloc,panic_abort -Z json-target-spec ) \
    || fail "cargo build failed"

RUST_LIB="${ACCEL_DIR}/target/${TARGET_NAME}/release/libopenvmaccel.a"
[ -f "${RUST_LIB}" ] || fail "${RUST_LIB} not produced"
cp "${RUST_LIB}" "${OUTPUT_DIR}/libopenvm.a"

# The accelerated k256/p256 pull in the whole `openvm` guest runtime, which
# brings its own entry point and libc-ish helpers. Rust emits one codegen unit
# per crate, so those ride along with any accelerator the guest references and
# collide with the guest's own _start (modules/zkvm_openvm/module.S), with
# musl's mem* and with rust_sys's allocator bridge. Making them local keeps the
# runtime's internal calls bound to its own copies and leaves the guest's
# versions the only globals the link can see.
#
# sys_alloc_aligned is subtler: bflat's rust_sys module wraps it
# (--wrap=sys_alloc_aligned) so OpenVM's bump allocator draws from pal's heap
# rather than opening a second one. Localizing the definition here keeps the
# wrap unambiguous.
LOCALIZE="_start __start memcpy memset memmove memcmp sys_alloc_aligned"
LOCALIZE_ARGS=""
for sym in ${LOCALIZE} ; do
    LOCALIZE_ARGS="${LOCALIZE_ARGS} --localize-symbol=${sym}"
done
echo "Localizing OpenVM runtime symbols:${LOCALIZE}"
"${OBJCOPY}" ${LOCALIZE_ARGS} "${OUTPUT_DIR}/libopenvm.a" || fail "objcopy failed"

# --- 2. Raw intrinsic shims ------------------------------------------------
echo "Assembling openvm_intrinsics.S..."
"${AS}" --march=rv64im --mabi=lp64 \
    "${ROOT_DIR}/src/openvm_intrinsics/openvm_intrinsics.S" \
    -o "${OUTPUT_DIR}/openvm_intrinsics.o" || fail "assembly failed"
# Clear the float-ABI marker so ld.lld accepts the object against the guest's
# soft-float crt1.o.
printf '\x00' | dd of="${OUTPUT_DIR}/openvm_intrinsics.o" bs=1 seek=48 count=1 conv=notrunc status=none

"${AR}" r "${OUTPUT_DIR}/libopenvm.a" "${OUTPUT_DIR}/openvm_intrinsics.o" || fail "ar failed"
"${RANLIB}" "${OUTPUT_DIR}/libopenvm.a" || fail "ranlib failed"

# --- 3. Prove the accelerators are really in there -------------------------
# custom-0 (0x0b) is keccak/sha2/u256; custom-1 (0x2b) is algebra/ecc, i.e. the
# curves. Zero of either means a silent fallback to portable Rust.
count_opcode() {
    riscv64-linux-gnu-objdump -d "$1" 2>/dev/null \
        | grep -oE '^ *[0-9a-f]+:\s+[0-9a-f]{8}' | awk '{print $2}' \
        | while read -r w ; do
            v=$((16#$w))
            [ $((v & 0x7f)) -eq "$2" ] && echo x
          done | wc -l
}
C0=$(count_opcode "${OUTPUT_DIR}/libopenvm.a" 11)
C1=$(count_opcode "${OUTPUT_DIR}/libopenvm.a" 43)
echo "OpenVM custom-0 instructions: ${C0}"
echo "OpenVM custom-1 instructions: ${C1}"
[ "${C0}" -gt 0 ] || fail "no custom-0 instructions - the hash accelerators did not activate"
[ "${C1}" -gt 0 ] || fail "no custom-1 instructions - the curve accelerators did not activate"

cp "${ROOT_DIR}/bflat-manifest.json" "${OUTPUT_DIR}/libopenvm.bflat.manifest"

echo "Build completed"
echo "Output: ${OUTPUT_DIR}/libopenvm.a"
echo "        ${OUTPUT_DIR}/libopenvm.bflat.manifest"
