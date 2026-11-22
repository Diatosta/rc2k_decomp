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

/// Loads and stores for the plain float words (`f32`, and `u32` / `i32` holding a float's bits)
/// the float code works on: the same moves as `CellX87`, on a value written through `&mut`.
pub trait PlainX87 {
    /// The word's float.
    fn ld(&self) -> X;
    /// `x` rounded to a float as the control word says, stored in the word.
    fn st(&mut self, x: X);
}

impl PlainX87 for f32 {
    fn ld(&self) -> X {
        bh::ld(self.to_bits())
    }

    fn st(&mut self, x: X) {
        *self = f32::from_bits(bh::st(x));
    }
}

impl PlainX87 for u32 {
    fn ld(&self) -> X {
        bh::ld(*self)
    }

    fn st(&mut self, x: X) {
        *self = bh::st(x);
    }
}

impl PlainX87 for i32 {
    fn ld(&self) -> X {
        bh::ld(*self as u32)
    }

    fn st(&mut self, x: X) {
        *self = bh::st(x) as i32;
    }
}
