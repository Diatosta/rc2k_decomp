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
    pub fn get(&self) -> [f32; N] {
        self.0.get().map(f32::from_bits)
    }

    /// The whole row put in, as bits.
    pub fn set_bits(&self, values: [u32; N]) {
        plain_written(&self.0);
        self.0.set(values);
    }

    /// The whole row put in.
    pub fn set(&self, values: [f32; N]) {
        self.set_bits(values.map(f32::to_bits));
    }

    /// Element `n`'s bits.
    pub fn bits_at(&self, n: usize) -> u32 {
        self.0.get()[n]
    }

    /// Element `n`.
    pub fn at(&self, n: usize) -> f32 {
        f32::from_bits(self.bits_at(n))
    }

    /// Element `n` loaded as the x87 loads a dword float.
    pub(crate) fn ld(&self, n: usize) -> crate::math::x87_float::X {
        crate::math::x87_float::ld(self.bits_at(n))
    }

    /// Element `n` put in, as bits.
    pub fn set_bits_at(&self, n: usize, bits: u32) {
        plain_written(&self.0);
        let mut values = self.0.get();
        values[n] = bits;
        self.0.set(values);
    }

    /// Element `n` put in.
    pub fn set_at(&self, n: usize, value: f32) {
        self.set_bits_at(n, value.to_bits());
    }

    /// Element `n` as a cell of its bits (for a table of the record's words).
    pub fn cell_at(&self, n: usize) -> &Cell<u32> {
        &self.cells()[n]
    }

    /// The elements as cells (the runs of the records' copies).
    pub(crate) fn cells(&self) -> &[Cell<u32>] {
        self.0.as_array_of_cells()
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
impl<const N: usize> crate::math::cells::Cells for PlainFloats<N> {
    fn bits(&self, n: usize) -> u32 {
        self.bits_at(n)
    }
    fn set_bits(&self, n: usize, bits: u32) {
        self.set_bits_at(n, bits);
    }
    fn count(&self) -> usize {
        N
    }
}

/// A snapshot's atomic row of floats takes the row's words (`race::frame_view`).
impl<const N: usize> SameShape<[crate::memory::session::Float; N]> for PlainFloats<N> {
    fn copy_cells_to(&self, to: &[crate::memory::session::Float; N]) {
        for (cell, bits) in to.iter().zip(self.bits()) {
            cell.set_bits(bits);
        }
    }

    fn merge_cells_to(&self, start: &Self, to: &[crate::memory::session::Float; N]) {
        let (now, start) = (self.bits(), start.bits());
        for n in 0..N {
            if now[n] != start[n] {
                to[n].set_bits(now[n]);
            }
        }
    }
}
