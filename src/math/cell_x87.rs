//! A float cell read and written as the x87 does: loaded and stored as a dword float.

use crate::math::x87_float::{self as bh, X};
use crate::memory::session::Cell32;

/// Loads and stores for the cells (`Float`, `Dword`) the float code works on.
pub trait CellX87 {
    /// The cell's float.
    fn ld(&self) -> X;
    /// `x` rounded to a float as the control word says, stored in the cell.
    fn st(&self, x: X);
}

// Through the cell's bits: the x87 load and store move the exact image (a NaN's payload, a
// signalling NaN's quiet bit as `x87_float` has it), which a Rust `f32` need not keep.
impl<T: Cell32 + ?Sized> CellX87 for T {
    fn ld(&self) -> X {
        bh::ld(self.bits())
    }

    fn st(&self, x: X) {
        self.set_bits(bh::st(x));
    }
}
