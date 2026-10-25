//! The memory places of the marks' x87 blocks, written as plain Rust: a block's operand is
//! the base address (or an index) its instructions used, `off` the displacement, so the same
//! place is read or written. Floats go through `x87_float`'s `X` (what the x87 holds).

use crate::math::x87_float::{self as bh, X};

/// `fld dword ptr [base + off]`.
pub(crate) unsafe fn ld(base: usize, off: usize) -> X {
    bh::ld(unsafe { (base.wrapping_add(off) as *const u32).read_unaligned() })
}

/// `fild dword ptr [base + off]`.
pub(crate) unsafe fn ldi(base: usize, off: usize) -> X {
    bh::int(unsafe { (base.wrapping_add(off) as *const i32).read_unaligned() })
}

/// `fstp dword ptr [base + off]` (the value, rounded to a float as the control word says).
pub(crate) unsafe fn st(base: usize, off: usize, x: X) {
    unsafe { (base.wrapping_add(off) as *mut u32).write_unaligned(bh::st(x)) };
}

/// `fistp dword ptr [base + off]`.
pub(crate) unsafe fn sti(base: usize, off: usize, x: X) {
    unsafe { (base.wrapping_add(off) as *mut i32).write_unaligned(bh::to_int(x)) };
}

/// The ten bytes `fld tbyte ptr [base + off]` reads.
pub(crate) unsafe fn ld80(base: usize, off: usize) -> [u8; 10] {
    unsafe { (base.wrapping_add(off) as *const [u8; 10]).read_unaligned() }
}

/// `fstp tbyte ptr [base + off]`.
pub(crate) unsafe fn st80(base: usize, off: usize, x: X) {
    unsafe { (base.wrapping_add(off) as *mut [u8; 10]).write_unaligned(bh::f80(x)) };
}

/// `fstsw word ptr [base + off]`: the compare bits C0, C2 and C3 (the other bits of the
/// status word are the stack's top and the sticky exceptions, which no reader looks at).
pub(crate) unsafe fn st16(base: usize, off: usize, status: u16) {
    unsafe { (base.wrapping_add(off) as *mut u16).write_unaligned(status) };
}

/// `mov reg, dword ptr [base + off]`.
pub(crate) unsafe fn rd(base: usize, off: usize) -> u32 {
    unsafe { (base.wrapping_add(off) as *const u32).read_unaligned() }
}

/// `mov dword ptr [base + off], reg`.
pub(crate) unsafe fn wr(base: usize, off: usize, v: u32) {
    unsafe { (base.wrapping_add(off) as *mut u32).write_unaligned(v) };
}
