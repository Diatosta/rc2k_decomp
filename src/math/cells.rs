//! A run of 32-bit words a helper reads (`Words`) or reads and writes (`Cells`) in place of an
//! address: a typed store's array (a matrix's cells, a car's velocity, a slice of one from a cell
//! on), a plain array of floats or words (read only), or `Off` (a run from its word `n` on).

use crate::memory::session::Cell32;

/// A run of 32-bit words read by index.
pub trait Words {
    /// Word `n`'s bits.
    fn bits(&self, n: usize) -> u32;
    /// Word `n` as the float it holds.
    fn float(&self, n: usize) -> f32 {
        f32::from_bits(self.bits(n))
    }
    /// How many words there are (unbounded for a run at an address).
    fn count(&self) -> usize {
        usize::MAX
    }
}

/// A run of 32-bit cells, read and written by index.
pub trait Cells: Words {
    /// Cell `n` set to `bits`.
    fn set_bits(&self, n: usize, bits: u32);
    /// Cell `n` set to the float `value`.
    fn set_float(&self, n: usize, value: f32) {
        self.set_bits(n, value.to_bits());
    }
}

/// A plain array of floats, read as their bits.
impl<const N: usize> Words for [f32; N] {
    fn bits(&self, n: usize) -> u32 {
        self[n].to_bits()
    }
    fn count(&self) -> usize {
        N
    }
}

/// A plain array of words.
impl<const N: usize> Words for [u32; N] {
    fn bits(&self, n: usize) -> u32 {
        self[n]
    }
    fn count(&self) -> usize {
        N
    }
}

/// A plain slice of floats, read as their bits.
impl Words for [f32] {
    fn bits(&self, n: usize) -> u32 {
        self[n].to_bits()
    }
    fn count(&self) -> usize {
        self.len()
    }
}

impl<C: Cell32> Words for [C] {
    fn bits(&self, n: usize) -> u32 {
        self[n].bits()
    }
    fn count(&self) -> usize {
        self.len()
    }
}

impl<C: Cell32> Cells for [C] {
    fn set_bits(&self, n: usize, bits: u32) {
        self[n].set_bits(bits);
    }
}

impl<C: Cell32, const N: usize> Words for [C; N] {
    fn bits(&self, n: usize) -> u32 {
        self[n].bits()
    }
    fn count(&self) -> usize {
        N
    }
}

impl<C: Cell32, const N: usize> Cells for [C; N] {
    fn set_bits(&self, n: usize, bits: u32) {
        self[n].set_bits(bits);
    }
}

/// A byte buffer's little-endian dwords.
impl<const N: usize> Words for crate::memory::session::Bytes<N> {
    fn bits(&self, n: usize) -> u32 {
        self.get_u32(4 * n)
    }
    fn count(&self) -> usize {
        N / 4
    }
}

impl<const N: usize> Cells for crate::memory::session::Bytes<N> {
    fn set_bits(&self, n: usize, bits: u32) {
        self.set_u32(4 * n, bits);
    }
}

/// A plain byte buffer's little-endian dwords.
impl<const N: usize> Words for crate::memory::session::PlainBytes<N> {
    fn bits(&self, n: usize) -> u32 {
        self.get_u32(4 * n)
    }
    fn count(&self) -> usize {
        N / 4
    }
}

impl<const N: usize> Cells for crate::memory::session::PlainBytes<N> {
    fn set_bits(&self, n: usize, bits: u32) {
        self.set_u32(4 * n, bits);
    }
}

/// A plain buffer's little-endian dwords.
impl<const N: usize> Words for crate::memory::plain_buffer::PlainBuffer<N> {
    fn bits(&self, n: usize) -> u32 {
        self.get_u32(4 * n)
    }
    fn count(&self) -> usize {
        N / 4
    }
}

impl<const N: usize> Cells for crate::memory::plain_buffer::PlainBuffer<N> {
    fn set_bits(&self, n: usize, bits: u32) {
        self.set_u32(4 * n, bits);
    }
}

impl<T: Words + ?Sized> Words for &T {
    fn bits(&self, n: usize) -> u32 {
        (**self).bits(n)
    }
    fn count(&self) -> usize {
        (**self).count()
    }
}

impl<T: Cells + ?Sized> Cells for &T {
    fn set_bits(&self, n: usize, bits: u32) {
        (**self).set_bits(n, bits);
    }
}

/// The first `N` cells' bits.
pub fn load<const N: usize>(cells: &(impl Words + ?Sized)) -> [u32; N] {
    std::array::from_fn(|n| cells.bits(n))
}

/// The first `n` cells of `from` into `to`, in order.
pub fn copy_cells(from: &(impl Words + ?Sized), to: &(impl Cells + ?Sized), n: usize) {
    for k in 0..n {
        to.set_bits(k, from.bits(k));
    }
}

/// `cells` from cell `from` on (a matrix's row).
pub struct Off<'a, T: Words + ?Sized>(pub &'a T, pub usize);

impl<T: Words + ?Sized> Words for Off<'_, T> {
    fn bits(&self, n: usize) -> u32 {
        self.0.bits(self.1 + n)
    }
}

impl<T: Cells + ?Sized> Cells for Off<'_, T> {
    fn set_bits(&self, n: usize, bits: u32) {
        self.0.set_bits(self.1 + n, bits);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::session::Float;

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
