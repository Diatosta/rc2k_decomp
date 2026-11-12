//! A run of 32-bit cells a helper reads and writes, in place of an address (phase 13, the
//! `Game` context: helpers that took a matrix's or a vector's address take its cells):
//! a typed store's array (`&MATRICES.turned_matrix`, `&CAR.velocity`, a slice of one from
//! a cell on), or `Off` (a run from its cell `n` on).

use crate::session::Cell32;

pub trait Cells {
    /// Cell `n`'s bits.
    fn bits(&self, n: usize) -> u32;
    /// Cell `n` set to `bits`.
    fn set_bits(&self, n: usize, bits: u32);
    /// Cell `n` as the float it holds.
    fn float(&self, n: usize) -> f32 {
        f32::from_bits(self.bits(n))
    }
    /// Cell `n` set to the float `value`.
    fn set_float(&self, n: usize, value: f32) {
        self.set_bits(n, value.to_bits());
    }
    /// How many cells there are (unbounded for a run at an address).
    fn count(&self) -> usize {
        usize::MAX
    }
}

impl<C: Cell32> Cells for [C] {
    fn bits(&self, n: usize) -> u32 {
        self[n].bits()
    }
    fn set_bits(&self, n: usize, bits: u32) {
        self[n].set_bits(bits);
    }
    fn count(&self) -> usize {
        self.len()
    }
}

impl<C: Cell32, const N: usize> Cells for [C; N] {
    fn bits(&self, n: usize) -> u32 {
        self[n].bits()
    }
    fn set_bits(&self, n: usize, bits: u32) {
        self[n].set_bits(bits);
    }
    fn count(&self) -> usize {
        N
    }
}

/// A byte buffer's little-endian dwords.
impl<const N: usize> Cells for crate::session::Bytes<N> {
    fn bits(&self, n: usize) -> u32 {
        self.get_u32(4 * n)
    }
    fn set_bits(&self, n: usize, bits: u32) {
        self.set_u32(4 * n, bits);
    }
    fn count(&self) -> usize {
        N / 4
    }
}

/// A plain byte buffer's little-endian dwords.
impl<const N: usize> Cells for crate::session::PlainBytes<N> {
    fn bits(&self, n: usize) -> u32 {
        self.get_u32(4 * n)
    }
    fn set_bits(&self, n: usize, bits: u32) {
        self.set_u32(4 * n, bits);
    }
    fn count(&self) -> usize {
        N / 4
    }
}

impl<T: Cells + ?Sized> Cells for &T {
    fn bits(&self, n: usize) -> u32 {
        (**self).bits(n)
    }
    fn set_bits(&self, n: usize, bits: u32) {
        (**self).set_bits(n, bits);
    }
    fn count(&self) -> usize {
        (**self).count()
    }
}

/// The first `N` cells' bits.
pub fn load<const N: usize>(cells: &(impl Cells + ?Sized)) -> [u32; N] {
    std::array::from_fn(|n| cells.bits(n))
}

/// The first `n` cells of `from` into `to`, in order (a `rep movsd`).
pub fn copy_cells(from: &(impl Cells + ?Sized), to: &(impl Cells + ?Sized), n: usize) {
    for k in 0..n {
        to.set_bits(k, from.bits(k));
    }
}

/// `cells` from cell `from` on (a matrix's row).
pub struct Off<'a, T: Cells + ?Sized>(pub &'a T, pub usize);

impl<T: Cells + ?Sized> Cells for Off<'_, T> {
    fn bits(&self, n: usize) -> u32 {
        self.0.bits(self.1 + n)
    }
    fn set_bits(&self, n: usize, bits: u32) {
        self.0.set_bits(self.1 + n, bits);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::Float;

    #[test]
    fn cells_by_index_and_from_a_cell() {
        let m: [Float; 4] = [
            Float::from_bits(0.0f32.to_bits()),
            Float::from_bits(1.0f32.to_bits()),
            Float::from_bits(2.0f32.to_bits()),
            Float::from_bits(3.0f32.to_bits()),
        ];
        assert_eq!(m.bits(2), 2.0f32.to_bits());
        Off(&m, 1).set_bits(1, 9.0f32.to_bits());
        assert_eq!(m[2].get(), 9.0);
        assert_eq!(m[1..].bits(0), 1.0f32.to_bits());
    }
}
