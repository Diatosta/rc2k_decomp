//! A row of floats a record keeps as one plain value (a matrix): its 32-bit words in one
//! `Cell`, read and written whole or by element, only by the game's thread.

use crate::memory::cells::{SameShape, ZeroCells};
use crate::memory::session::plain_written;
use std::cell::Cell;

/// `N` floats held by their bits in one cell.
#[repr(transparent)]
pub struct PlainFloats<const N: usize>(Cell<[u32; N]>);

impl<const N: usize> PlainFloats<N> {
    /// A row holding the floats whose bits are `values`.
    pub const fn from_bits(values: [u32; N]) -> Self {
        Self(Cell::new(values))
    }

    /// The row's bits.
    pub fn bits(&self) -> [u32; N] {
        self.0.get()
    }

    /// The row's floats.
    #[cfg(test)]
    pub fn get(&self) -> [f32; N] {
        self.0.get().map(f32::from_bits)
    }

    /// The whole row put in.
    #[cfg(test)]
    pub fn set(&self, values: [f32; N]) {
        self.set_bits(values.map(f32::to_bits));
    }

    /// The whole row put in, as bits.
    pub fn set_bits(&self, values: [u32; N]) {
        plain_written(&self.0);
        self.0.set(values);
    }

    /// Element `n`'s bits.
    pub fn bits_at(&self, n: usize) -> u32 {
        self.0.get()[n]
    }

    /// Element `n` put in, as bits.
    pub fn set_bits_at(&self, n: usize, bits: u32) {
        plain_written(&self.0);
        let mut values = self.0.get();
        values[n] = bits;
        self.0.set(values);
    }

    /// The elements as cells (the runs of the records' copies).
    pub(crate) fn cells(&self) -> &[Cell<u32>] {
        self.0.as_array_of_cells()
    }
}

/// The row as the one run of its elements (a car's overrun lists).
impl<const N: usize> crate::memory::session::Run for PlainFloats<N> {
    fn count(&self) -> usize {
        N
    }
    fn bits(&self, i: usize) -> u32 {
        self.bits_at(i)
    }
    fn set_bits(&self, i: usize, v: u32) {
        self.set_bits_at(i, v);
    }
}

impl<const N: usize> ZeroCells for PlainFloats<N> {
    const ZERO: Self = Self::from_bits([0; N]);
}

impl<const N: usize> SameShape<PlainFloats<N>> for PlainFloats<N> {
    fn copy_cells_to(&self, to: &PlainFloats<N>) {
        to.set_bits(self.bits());
    }

    fn merge_cells_to(&self, start: &Self, to: &PlainFloats<N>) {
        let (now, start) = (self.bits(), start.bits());
        for n in 0..N {
            if now[n] != start[n] {
                to.set_bits_at(n, now[n]);
            }
        }
    }
}

/// The row's elements by index, as the helpers on runs of cells take them.
impl<const N: usize> crate::math::cells::Words for PlainFloats<N> {
    fn bits(&self, n: usize) -> u32 {
        self.bits_at(n)
    }
    fn count(&self) -> usize {
        N
    }
}

impl<const N: usize> crate::math::cells::Cells for PlainFloats<N> {
    fn set_bits(&self, n: usize, bits: u32) {
        self.set_bits_at(n, bits);
    }
}

/// A snapshot's atomic row of words takes the row's words (`race::frame_view`).
impl<C: crate::memory::session::Cell32, const N: usize> SameShape<[C; N]> for PlainFloats<N> {
    fn copy_cells_to(&self, to: &[C; N]) {
        for (cell, bits) in to.iter().zip(self.bits()) {
            cell.set_bits(bits);
        }
    }

    fn merge_cells_to(&self, start: &Self, to: &[C; N]) {
        let (now, start) = (self.bits(), start.bits());
        for n in 0..N {
            if now[n] != start[n] {
                to[n].set_bits(now[n]);
            }
        }
    }
}

/// Reads of a row of floats held as a plain array (a car's matrix or vector) by element.
pub trait FloatRow {
    /// Element `n`'s bits.
    fn bits_at(&self, n: usize) -> u32;
    /// Element `n`.
    fn at(&self, n: usize) -> f32;
    /// Element `n` loaded as the x87 loads a dword float.
    fn ld(&self, n: usize) -> crate::math::x87_float::X;
}

impl<const N: usize> FloatRow for [f32; N] {
    fn bits_at(&self, n: usize) -> u32 {
        self[n].to_bits()
    }
    fn at(&self, n: usize) -> f32 {
        self[n]
    }
    fn ld(&self, n: usize) -> crate::math::x87_float::X {
        crate::math::x87_float::ld(self[n].to_bits())
    }
}

/// Writes of a row of floats held as a plain array by element.
pub trait FloatRowMut {
    /// Element `n` put in, as bits.
    fn set_bits_at(&mut self, n: usize, bits: u32);
    /// Element `n` put in.
    fn set_at(&mut self, n: usize, value: f32);
    /// `x` rounded to a float as the control word says, put in as element `n`.
    fn st_at(&mut self, n: usize, x: crate::math::x87_float::X);
}

impl<const N: usize> FloatRowMut for [f32; N] {
    fn set_bits_at(&mut self, n: usize, bits: u32) {
        self[n] = f32::from_bits(bits);
    }
    fn set_at(&mut self, n: usize, value: f32) {
        self[n] = value;
    }
    fn st_at(&mut self, n: usize, x: crate::math::x87_float::X) {
        self[n] = f32::from_bits(crate::math::x87_float::st(x));
    }
}
