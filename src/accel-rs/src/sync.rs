//! GCC `__sync_*` builtins for a single-hart guest.
//!
//! OpenVM decodes RV64IM - no A extension - so LLVM cannot lower an atomic
//! operation to `lr`/`sc`/`amo*`. Two things follow from the target spec:
//!
//! - `+forced-atomics` tells LLVM that native-width atomics exist, which turns
//!   plain atomic loads and stores into plain loads and stores plus a `fence`
//!   (OpenVM's decoder accepts `fence`; it rejects `fence.i`). Without it LLVM
//!   emits `__atomic_*` calls into a libatomic that does not exist here.
//! - What it cannot inline - compare-and-swap and the read-modify-write
//!   operations - becomes a `__sync_*` call instead. `compiler_builtins`
//!   implements those only for ARM Thumb, so on RISC-V nothing defines them
//!   and the guest fails to link.
//!
//! Hence this module. The implementations are deliberately NOT atomic: an
//! OpenVM guest is single-hart with no interrupts and no thread creation
//! instruction, so no other agent can observe or interleave with the
//! read-modify-write. Under that assumption a plain load, compare and store is
//! indistinguishable from the atomic sequence - which is exactly the reasoning
//! behind the `singlethread` flag in the target spec.
//!
//! The full width and operation set is provided, not just what today's
//! dependency graph happens to reach: `once_cell`'s spin-free `race` module is
//! what pulls these in through the accelerated k256/p256 curves, and which
//! width it lands on is an implementation detail of a crate we do not control.
//!
//! Volatile accesses keep the sequence intact so that a caller that inspects
//! the location through another pointer still sees the write.

macro_rules! sync_ops {
    ($ty:ty, $suffix:literal,
     $val_cas:ident, $bool_cas:ident, $test_and_set:ident,
     $add:ident, $sub:ident, $and:ident, $or:ident, $xor:ident, $nand:ident) => {
        #[doc = concat!("`__sync_val_compare_and_swap_", $suffix, "`")]
        #[no_mangle]
        pub unsafe extern "C" fn $val_cas(ptr: *mut $ty, old: $ty, new: $ty) -> $ty {
            let current = ptr.read_volatile();
            if current == old {
                ptr.write_volatile(new);
            }
            current
        }

        #[doc = concat!("`__sync_bool_compare_and_swap_", $suffix, "`")]
        #[no_mangle]
        pub unsafe extern "C" fn $bool_cas(ptr: *mut $ty, old: $ty, new: $ty) -> bool {
            let current = ptr.read_volatile();
            if current == old {
                ptr.write_volatile(new);
                true
            } else {
                false
            }
        }

        #[doc = concat!("`__sync_lock_test_and_set_", $suffix, "`")]
        #[no_mangle]
        pub unsafe extern "C" fn $test_and_set(ptr: *mut $ty, value: $ty) -> $ty {
            let current = ptr.read_volatile();
            ptr.write_volatile(value);
            current
        }

        sync_rmw!($ty, $add, |a: $ty, b: $ty| a.wrapping_add(b));
        sync_rmw!($ty, $sub, |a: $ty, b: $ty| a.wrapping_sub(b));
        sync_rmw!($ty, $and, |a: $ty, b: $ty| a & b);
        sync_rmw!($ty, $or, |a: $ty, b: $ty| a | b);
        sync_rmw!($ty, $xor, |a: $ty, b: $ty| a ^ b);
        sync_rmw!($ty, $nand, |a: $ty, b: $ty| !(a & b));
    };
}

/// Read-modify-write returning the value as it was *before* the operation,
/// which is what the `__sync_fetch_and_*` family is specified to do.
macro_rules! sync_rmw {
    ($ty:ty, $name:ident, $op:expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(ptr: *mut $ty, value: $ty) -> $ty {
            let current = ptr.read_volatile();
            ptr.write_volatile($op(current, value));
            current
        }
    };
}

sync_ops!(
    u8, "1",
    __sync_val_compare_and_swap_1, __sync_bool_compare_and_swap_1, __sync_lock_test_and_set_1,
    __sync_fetch_and_add_1, __sync_fetch_and_sub_1, __sync_fetch_and_and_1,
    __sync_fetch_and_or_1, __sync_fetch_and_xor_1, __sync_fetch_and_nand_1
);
sync_ops!(
    u16, "2",
    __sync_val_compare_and_swap_2, __sync_bool_compare_and_swap_2, __sync_lock_test_and_set_2,
    __sync_fetch_and_add_2, __sync_fetch_and_sub_2, __sync_fetch_and_and_2,
    __sync_fetch_and_or_2, __sync_fetch_and_xor_2, __sync_fetch_and_nand_2
);
sync_ops!(
    u32, "4",
    __sync_val_compare_and_swap_4, __sync_bool_compare_and_swap_4, __sync_lock_test_and_set_4,
    __sync_fetch_and_add_4, __sync_fetch_and_sub_4, __sync_fetch_and_and_4,
    __sync_fetch_and_or_4, __sync_fetch_and_xor_4, __sync_fetch_and_nand_4
);
sync_ops!(
    u64, "8",
    __sync_val_compare_and_swap_8, __sync_bool_compare_and_swap_8, __sync_lock_test_and_set_8,
    __sync_fetch_and_add_8, __sync_fetch_and_sub_8, __sync_fetch_and_and_8,
    __sync_fetch_and_or_8, __sync_fetch_and_xor_8, __sync_fetch_and_nand_8
);

/// `__sync_synchronize` - a full barrier. Nothing to order on one hart.
#[no_mangle]
pub extern "C" fn __sync_synchronize() {}
