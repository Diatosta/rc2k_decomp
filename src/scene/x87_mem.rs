//! Memory operands of the blocks that were x87 assembly, as plain functions: a base pointer
//! and a byte offset, unaligned, in the `X` model of `math/x87_float.rs`.
//!
//! `f` is `fld dword`, `i` is `fild dword`, `t80` is `fld tbyte`; `wf` is `fstp dword`,
//! `wi` is `fistp dword`, `wt80` is `fstp tbyte`, `wh` stores a status word.

use crate::math::x87_float::{X, f80, from_f80, int, ld, st, to_int};

/// `fld dword ptr [base + off]`.
pub(crate) unsafe fn fld(base: *const u8, off: usize) -> X {
    ld(unsafe { base.add(off).cast::<u32>().read_unaligned() })
}

/// `fild dword ptr [base + off]`.
pub(crate) unsafe fn fild(base: *const u8, off: usize) -> X {
    int(unsafe { base.add(off).cast::<i32>().read_unaligned() })
}

/// `fld tbyte ptr [base + off]`.
pub(crate) unsafe fn fld80(base: *const u8, off: usize) -> X {
    from_f80(unsafe { base.add(off).cast::<[u8; 10]>().read_unaligned() })
}

/// `fstp dword ptr [base + off]`: the value rounded to a float as the control word says.
pub(crate) unsafe fn fstp(base: *mut u8, off: usize, x: X) {
    unsafe { base.add(off).cast::<u32>().write_unaligned(st(x)) }
}

/// `fistp dword ptr [base + off]`: the value rounded to a whole number as the control word
/// says.
pub(crate) unsafe fn fistp(base: *mut u8, off: usize, x: X) {
    unsafe { base.add(off).cast::<i32>().write_unaligned(to_int(x)) }
}

/// `fstp tbyte ptr [base + off]`.
pub(crate) unsafe fn fstp80(base: *mut u8, off: usize, x: X) {
    unsafe { base.add(off).cast::<[u8; 10]>().write_unaligned(f80(x)) }
}

/// `fnstsw word ptr [base + off]`.
pub(crate) unsafe fn fnstsw(base: *mut u8, off: usize, status: u16) {
    unsafe { base.add(off).cast::<u16>().write_unaligned(status) }
}

/// `mov dword ptr [base + off], value`.
pub(crate) unsafe fn mov(base: *mut u8, off: usize, value: u32) {
    unsafe { base.add(off).cast::<u32>().write_unaligned(value) }
}
