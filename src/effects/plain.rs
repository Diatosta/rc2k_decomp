//! The memory places of the marks' x87 blocks, written as plain Rust: typed cells and
//! buffers, read and written where the blocks' instructions did. Floats go through
//! `x87_float`'s `X` (what the x87 holds).

use crate::math::x87_float::{self as bh, X};

// The three cell helpers go by bits: the x87 loads and stores the exact image, `ldi_cell`
// reads a whole number held in the cell.

/// A float cell (a typed record's field) loaded as the x87 loads a dword float.
pub(crate) fn ld_cell(cell: &impl crate::session::Cell32) -> X {
    bh::ld(cell.bits())
}

/// A dword cell (a typed record's field) loaded as a whole number.
pub(crate) fn ldi_cell(cell: &impl crate::session::Cell32) -> X {
    bh::int(cell.bits() as i32)
}

/// Stores into a float cell (a typed record's field) as the x87 stores a dword float.
pub(crate) fn st_cell(cell: &impl crate::session::Cell32, x: X) {
    cell.set_bits(bh::st(x));
}

/// `len` bytes at byte `offset` of a buffer; past its end the program ends naming it.
fn bytes_at(bytes: &impl crate::session::ByteBuffer, offset: usize, len: usize) -> Vec<u8> {
    bytes
        .read_bytes_at(offset, len, "effects::plain's operand")
        .unwrap_or_else(|e| crate::fatal::raise::end("effects::plain's operand", e))
}

/// `data` written at byte `offset` of a buffer; past its end the program ends naming it.
fn put_at(bytes: &impl crate::session::ByteBuffer, offset: usize, data: &[u8]) {
    bytes
        .write_bytes_at(offset, data, "effects::plain's operand")
        .unwrap_or_else(|e| crate::fatal::raise::end("effects::plain's operand", e));
}

/// The dword at byte `offset` of a buffer, loaded as a float.
pub(crate) fn ld_at(bytes: &impl crate::session::ByteBuffer, offset: usize) -> X {
    let b = bytes_at(bytes, offset, 4);
    bh::ld(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}

/// Stores as a float into the dword at byte `offset` of a buffer.
pub(crate) fn st_at(bytes: &impl crate::session::ByteBuffer, offset: usize, x: X) {
    put_at(bytes, offset, &bh::st(x).to_le_bytes());
}

/// The ten bytes (an 80-bit float) at byte `offset` of a buffer.
pub(crate) fn ld80_at(bytes: &impl crate::session::ByteBuffer, offset: usize) -> [u8; 10] {
    let b = bytes_at(bytes, offset, 10);
    std::array::from_fn(|i| b[i])
}

/// Writes ten bytes (an 80-bit float) at byte `offset` of a buffer.
pub(crate) fn st80_at(bytes: &impl crate::session::ByteBuffer, offset: usize, x: X) {
    put_at(bytes, offset, &bh::f80(x));
}

/// The status word's compare bits C0, C2 and C3 (the other bits are the stack's top and the
/// sticky exceptions, which no reader looks at) into the low half of
/// `MarksWork.fpu_status` (its high half kept).
pub(crate) fn set_status(work: &crate::effects::work_store::MarksWork, status: u16) {
    let s = &work.fpu_status;
    s.set((s.get() & 0xFFFF_0000) | u32::from(status));
}
