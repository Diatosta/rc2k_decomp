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

/// `fld dword ptr` of a float cell (a typed record's field).
pub(crate) fn ld_cell(cell: &crate::session::Float) -> X {
    bh::ld(cell.bits())
}

/// `fild dword ptr` of a dword cell (a typed record's field).
pub(crate) fn ldi_cell(cell: &crate::session::Dword) -> X {
    bh::int(cell.get() as i32)
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

/// `fstp dword ptr` into a float cell (a typed record's field).
pub(crate) fn st_cell(cell: &crate::session::Float, x: X) {
    cell.set_bits(bh::st(x));
}

/// `len` bytes at byte `offset` of a buffer; past its end the program ends naming it, as
/// the operands by address did (`pointers::read_or_end`).
fn bytes_at<const N: usize>(
    bytes: &crate::session::Bytes<N>,
    offset: usize,
    len: usize,
) -> Vec<u8> {
    bytes
        .read_at(offset, len, "effects::plain's operand")
        .unwrap_or_else(|e| crate::fatal::raise::end("effects::plain's operand", e))
}

/// `data` written at byte `offset` of a buffer; past its end the program ends naming it.
fn put_at<const N: usize>(bytes: &crate::session::Bytes<N>, offset: usize, data: &[u8]) {
    bytes
        .write_at(offset, data, "effects::plain's operand")
        .unwrap_or_else(|e| crate::fatal::raise::end("effects::plain's operand", e));
}

/// `fld dword ptr` of the dword at byte `offset` of a buffer.
pub(crate) fn ld_at<const N: usize>(bytes: &crate::session::Bytes<N>, offset: usize) -> X {
    let b = bytes_at(bytes, offset, 4);
    bh::ld(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// `fstp dword ptr` into the dword at byte `offset` of a buffer.
pub(crate) fn st_at<const N: usize>(bytes: &crate::session::Bytes<N>, offset: usize, x: X) {
    put_at(bytes, offset, &bh::st(x).to_le_bytes());
}

/// The ten bytes `fld tbyte ptr` reads at byte `offset` of a buffer.
pub(crate) fn ld80_at<const N: usize>(bytes: &crate::session::Bytes<N>, offset: usize) -> [u8; 10] {
    let b = bytes_at(bytes, offset, 10);
    std::array::from_fn(|i| b[i])
}

/// `fstp tbyte ptr` at byte `offset` of a buffer.
pub(crate) fn st80_at<const N: usize>(bytes: &crate::session::Bytes<N>, offset: usize, x: X) {
    put_at(bytes, offset, &bh::f80(x));
}

/// The ten bytes `fld tbyte ptr [base + off]` reads.
pub(crate) fn ld80(base: usize, off: usize) -> [u8; 10] {
    crate::pointers::read_or_end(base.wrapping_add(off) as u32, "a ten-byte float")
}

/// `fstp tbyte ptr [base + off]`.
pub(crate) fn st80(base: usize, off: usize, x: X) {
    crate::pointers::write_or_end(
        base.wrapping_add(off) as u32,
        bh::f80(x),
        "a ten-byte float",
    );
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
