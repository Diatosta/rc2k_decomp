//! Memory operands of the blocks that were x87 assembly, as plain functions: a base pointer
//! and a byte offset, unaligned, in the `X` model of `math/x87_float.rs`.
//!
//! `f` is `fld dword`, `i` is `fild dword`, `t80` is `fld tbyte`; `wf` is `fstp dword`,
//! `wi` is `fistp dword`, `wt80` is `fstp tbyte`, `wh` stores a status word.

use crate::math::x87_float::{X, f80, from_f80, int, ld, st, to_int};

/// An operand's base: an address, as the runs hold it, or a pointer.
pub(crate) trait Base: Copy {
    fn address(self) -> usize;
}

impl Base for usize {
    fn address(self) -> usize {
        self
    }
}

impl Base for *const u8 {
    fn address(self) -> usize {
        self as usize
    }
}

impl Base for *mut u8 {
    fn address(self) -> usize {
        self as usize
    }
}

/// The operand's address, its `len` bytes checked to lie in one region (the program ends
/// otherwise, `pointers::check_or_end`).
fn operand(base: impl Base, off: usize, len: usize) -> *mut u8 {
    let at = base.address().wrapping_add(off);
    crate::pointers::check_or_end(at as u32, len, "an x87 operand");
    at as *mut u8
}

/// `fld dword ptr [base + off]`.
pub(crate) fn fld(base: impl Base, off: usize) -> X {
    // SAFETY: checked in one live region.
    ld(unsafe { operand(base, off, 4).cast::<u32>().read_unaligned() })
}

/// `fild dword ptr [base + off]`.
pub(crate) fn fild(base: impl Base, off: usize) -> X {
    // SAFETY: checked in one live region.
    int(unsafe { operand(base, off, 4).cast::<i32>().read_unaligned() })
}

/// `fld tbyte ptr [base + off]`.
pub(crate) fn fld80(base: impl Base, off: usize) -> X {
    // SAFETY: checked in one live region.
    from_f80(unsafe { operand(base, off, 10).cast::<[u8; 10]>().read_unaligned() })
}

/// `fstp dword ptr [base + off]`: the value rounded to a float as the control word says.
pub(crate) fn fstp(base: impl Base, off: usize, x: X) {
    // SAFETY: checked in one live region.
    unsafe { operand(base, off, 4).cast::<u32>().write_unaligned(st(x)) }
}

/// `fistp dword ptr [base + off]`: the value rounded to a whole number as the control word
/// says.
pub(crate) fn fistp(base: impl Base, off: usize, x: X) {
    // SAFETY: checked in one live region.
    unsafe {
        operand(base, off, 4)
            .cast::<i32>()
            .write_unaligned(to_int(x))
    }
}

/// `fstp tbyte ptr [base + off]`.
pub(crate) fn fstp80(base: impl Base, off: usize, x: X) {
    // SAFETY: checked in one live region.
    unsafe {
        operand(base, off, 10)
            .cast::<[u8; 10]>()
            .write_unaligned(f80(x))
    }
}

/// `fnstsw word ptr [base + off]`.
pub(crate) fn fnstsw(base: impl Base, off: usize, status: u16) {
    // SAFETY: checked in one live region.
    unsafe { operand(base, off, 2).cast::<u16>().write_unaligned(status) }
}

/// `mov dword ptr [base + off], value`.
pub(crate) fn mov(base: impl Base, off: usize, value: u32) {
    // SAFETY: checked in one live region.
    unsafe { operand(base, off, 4).cast::<u32>().write_unaligned(value) }
}
