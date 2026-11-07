//! `scene::adjugate_matrix` (0x4BCF6B): a matrix's turn undone without the division: its
//! 3 x 3 part's cofactors, transposed (rows 0x10 bytes apart), and its place (+0x30 ..
//! +0x38) negated, not turned.
//!
//! Takes: the matrix, where the result goes (the matrix itself allowed)
//!
//! Each of the nine a product less a product (`x87::product_difference`), written in the
//! original's order and read from memory as it then is (in place, later ones see earlier
//! results). +0xC, +0x1C, +0x2C, +0x3C are not written. One caller, `sub_4C9AF6`. Refuses a
//! null matrix or result. The Rust version is `adjugate_matrix_plain`; the x87's stays for
//! `effects::cars_particles`.

use crate::math::cells::Cells;
use crate::math::x87;

/// The Rust version (see the header), the x87's: for `effects::cars_particles`
/// (unconverted). The cells are the matrix's and the result's from their first dword on.
pub fn adjugate_matrix(matrix: &(impl Cells + ?Sized), out: &(impl Cells + ?Sized)) {
    use rc2k_core::scene::matrices::COFACTORS;
    let get = |o: u32| matrix.bits(o as usize / 4);
    let put = |o: u32, v: u32| out.set_bits(o as usize / 4, v);
    for (to, [a, b, c, d]) in COFACTORS {
        put(to, x87::product_difference(get(a), get(b), get(c), get(d)));
    }
    for o in [0x30, 0x34, 0x38] {
        put(o, x87::negated(get(o)));
    }
}
