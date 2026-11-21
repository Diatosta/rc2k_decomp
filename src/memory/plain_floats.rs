//! A row of floats a record keeps as one plain value (a matrix): its 32-bit words in one
//! `Cell`, read and written whole or by element, only by the game's thread.

use crate::memory::cells::{SameShape, ZeroCells};
use crate::memory::session::{Cell32, plain_written};
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

    /// `x` rounded to a float as the control word says, put in as element `n`.
    pub(crate) fn st_at(&self, n: usize, x: crate::math::x87_float::X) {
        self.set_bits_at(n, crate::math::x87_float::st(x));
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

    /// Element `n` as a word whose writes are checked like the row's own (for a table of the
    /// record's words).
    pub fn cell_at(&self, n: usize) -> PlainWord<'_> {
        PlainWord(&self.cells()[n])
    }

    /// The elements as float places, in order: what the loops over a gauge row and the
    /// destructuring of one take.
    pub fn floats(&self) -> [FloatAt<'_>; N] {
        std::array::from_fn(|n| self.float_at(n))
    }

    /// Element `n` as a float place.
    pub fn float_at(&self, n: usize) -> FloatAt<'_> {
        FloatAt(self.cell_at(n))
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

/// One element of a row as a cell of its bits; writing it is as checked as writing the row.
#[derive(Clone, Copy)]
pub struct PlainWord<'a>(&'a Cell<u32>);

#[cfg(test)]
impl PlainWord<'_> {
    /// The element's place in memory.
    pub fn addr(&self) -> usize {
        self.0.as_ptr().addr()
    }
}

impl crate::memory::session::Cell32 for PlainWord<'_> {
    fn bits(&self) -> u32 {
        self.0.get()
    }
    fn set_bits(&self, bits: u32) {
        plain_written(self.0);
        self.0.set(bits);
    }
}

/// One element of a row of floats as a float cell; writing it is as checked as writing the row.
#[derive(Clone, Copy)]
pub struct FloatAt<'a>(PlainWord<'a>);

impl crate::memory::session::Cell32 for FloatAt<'_> {
    fn bits(&self) -> u32 {
        self.0.bits()
    }
    fn set_bits(&self, bits: u32) {
        self.0.set_bits(bits);
    }
}

impl FloatAt<'_> {
    /// The element's float.
    pub fn get(&self) -> f32 {
        f32::from_bits(self.0.bits())
    }

    /// Puts the float in.
    pub fn set(&self, value: f32) {
        self.0.set_bits(value.to_bits());
    }

    /// The element's place in memory.
    #[cfg(test)]
    pub fn addr(&self) -> usize {
        self.0.addr()
    }
}

impl crate::memory::session::FloatLike for FloatAt<'_> {
    fn get(&self) -> f32 {
        f32::from_bits(self.0.bits())
    }
    fn set(&self, value: f32) {
        self.0.set_bits(value.to_bits());
    }
}

/// A word of a record by reference, or one element of a row: what a table of words holds.
#[derive(Clone, Copy)]
pub enum WordRef<'a> {
    Cell(&'a dyn crate::memory::session::Cell32),
    Element(PlainWord<'a>),
}

#[cfg(test)]
impl WordRef<'_> {
    /// The word's place in memory.
    pub fn addr(&self) -> usize {
        match self {
            Self::Cell(cell) => core::ptr::from_ref(*cell).addr(),
            Self::Element(word) => word.addr(),
        }
    }
}

impl crate::memory::session::Cell32 for WordRef<'_> {
    fn bits(&self) -> u32 {
        match self {
            Self::Cell(cell) => cell.bits(),
            Self::Element(word) => word.bits(),
        }
    }
    fn set_bits(&self, bits: u32) {
        match self {
            Self::Cell(cell) => cell.set_bits(bits),
            Self::Element(word) => word.set_bits(bits),
        }
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

/// A float cell of a record, a whole cell or one element of a row: what a table of float fields
/// holds.
#[derive(Clone, Copy)]
pub enum FloatRef<'a> {
    Cell(&'a crate::memory::session::PlainFloat),
    Element(FloatAt<'a>),
}

impl<'a> From<&'a crate::memory::session::PlainFloat> for FloatRef<'a> {
    fn from(cell: &'a crate::memory::session::PlainFloat) -> Self {
        Self::Cell(cell)
    }
}

impl<'a> From<FloatAt<'a>> for FloatRef<'a> {
    fn from(at: FloatAt<'a>) -> Self {
        Self::Element(at)
    }
}

impl FloatRef<'_> {
    /// The float.
    pub fn get(&self) -> f32 {
        f32::from_bits(self.bits())
    }

    /// Puts the float in.
    pub fn set(&self, value: f32) {
        self.set_bits(value.to_bits());
    }

    /// The place in memory.
    #[cfg(test)]
    pub fn addr(&self) -> usize {
        match self {
            Self::Cell(cell) => core::ptr::from_ref(*cell).addr(),
            Self::Element(at) => at.addr(),
        }
    }
}

impl Cell32 for FloatRef<'_> {
    fn bits(&self) -> u32 {
        match self {
            Self::Cell(cell) => cell.bits(),
            Self::Element(at) => at.bits(),
        }
    }
    fn set_bits(&self, bits: u32) {
        match self {
            Self::Cell(cell) => cell.set_bits(bits),
            Self::Element(at) => at.set_bits(bits),
        }
    }
}

impl crate::memory::session::FloatLike for FloatRef<'_> {
    fn get(&self) -> f32 {
        f32::from_bits(self.bits())
    }
    fn set(&self, value: f32) {
        self.set_bits(value.to_bits());
    }
}
