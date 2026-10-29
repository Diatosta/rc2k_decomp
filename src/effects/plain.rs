//! The memory places of the marks' x87 blocks, written as plain Rust: a block's operand is
//! the base address (or an index) its instructions used, `off` the displacement, so the same
//! place is read or written. Floats go through `x87_float`'s `X` (what the x87 holds).

use crate::math::x87_float::{self as bh, X};

/// `fld dword ptr [base + off]`.
pub(crate) fn ld(base: usize, off: usize) -> X {
    bh::ld(crate::pointers::read_or_end::<u32>(
        (base.wrapping_add(off)) as u32,
        "effects::plain's operand",
    ))
}

/// `fild dword ptr [base + off]`.
pub(crate) fn ldi(base: usize, off: usize) -> X {
    bh::int(crate::pointers::read_or_end::<i32>(
        (base.wrapping_add(off)) as u32,
        "effects::plain's operand",
    ))
}

/// `fstp dword ptr [base + off]` (the value, rounded to a float as the control word says).
pub(crate) fn st(base: usize, off: usize, x: X) {
    crate::pointers::write_or_end::<u32>(
        (base.wrapping_add(off)) as u32,
        bh::st(x),
        "effects::plain's operand",
    );
}

/// `fistp dword ptr [base + off]`.
pub(crate) fn sti(base: usize, off: usize, x: X) {
    crate::pointers::write_or_end::<i32>(
        (base.wrapping_add(off)) as u32,
        bh::to_int(x),
        "effects::plain's operand",
    );
}

/// The ten bytes `fld tbyte ptr [base + off]` reads.
pub(crate) fn ld80(base: usize, off: usize) -> [u8; 10] {
    unsafe {
        (crate::pointers::host_at(base.wrapping_add(off)) as *const [u8; 10]).read_unaligned()
    }
}

/// `fstp tbyte ptr [base + off]`.
pub(crate) fn st80(base: usize, off: usize, x: X) {
    unsafe {
        (crate::pointers::host_at(base.wrapping_add(off)) as *mut [u8; 10])
            .write_unaligned(bh::f80(x))
    };
}

/// `fstsw word ptr [base + off]`: the compare bits C0, C2 and C3 (the other bits of the
/// status word are the stack's top and the sticky exceptions, which no reader looks at).
/// `fstsw word ptr [g_fpuStatus]`: the status's compare bits into the low half of
/// `MARKS_WORK.fpu_status` (its high half kept).
pub(crate) fn set_status(status: u16) {
    let s = &crate::effects::work_store::MARKS_WORK.fpu_status;
    s.set_bits((s.bits() & 0xFFFF_0000) | u32::from(status));
}

pub(crate) fn st16(base: usize, off: usize, status: u16) {
    crate::pointers::write_or_end::<u16>(
        (base.wrapping_add(off)) as u32,
        status,
        "effects::plain's operand",
    );
}
