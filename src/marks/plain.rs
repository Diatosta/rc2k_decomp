//! The memory places of the marks' x87 blocks, written as plain Rust: a block's operand is
//! the base address (or an index) its instructions used, `off` the displacement, so the same
//! place is read or written. Floats go through `x87_float`'s `X` (what the x87 holds).

use crate::math::x87_float::{self as bh, X};

/// `fld dword ptr [base + off]`.
pub(crate) fn ld(base: usize, off: usize) -> X {
    bh::ld(unsafe { (base.wrapping_add(off) as *const u32).read_unaligned() })
}

/// `fild dword ptr [base + off]`.
pub(crate) fn ldi(base: usize, off: usize) -> X {
    bh::int(unsafe { (base.wrapping_add(off) as *const i32).read_unaligned() })
}

/// `fstp dword ptr [base + off]` (the value, rounded to a float as the control word says).
pub(crate) fn st(base: usize, off: usize, x: X) {
    unsafe { (base.wrapping_add(off) as *mut u32).write_unaligned(bh::st(x)) };
}

/// `fistp dword ptr [base + off]`.
pub(crate) fn sti(base: usize, off: usize, x: X) {
    unsafe { (base.wrapping_add(off) as *mut i32).write_unaligned(bh::to_int(x)) };
}

/// The ten bytes `fld tbyte ptr [base + off]` reads.
pub(crate) fn ld80(base: usize, off: usize) -> [u8; 10] {
    unsafe { (base.wrapping_add(off) as *const [u8; 10]).read_unaligned() }
}

/// `fstp tbyte ptr [base + off]`.
pub(crate) fn st80(base: usize, off: usize, x: X) {
    unsafe { (base.wrapping_add(off) as *mut [u8; 10]).write_unaligned(bh::f80(x)) };
}

/// `fstsw word ptr [base + off]`: the compare bits C0, C2 and C3 (the other bits of the
/// status word are the stack's top and the sticky exceptions, which no reader looks at).
pub(crate) fn st16(base: usize, off: usize, status: u16) {
    unsafe { (base.wrapping_add(off) as *mut u16).write_unaligned(status) };
}
