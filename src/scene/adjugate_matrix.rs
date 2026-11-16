//! `scene::adjugate_matrix`: a matrix's turn undone without the division: its 3 x 3 part's
//! cofactors, transposed (rows 0x10 bytes apart), and its place (+0x30 .. +0x38) negated, not
//! turned. The matrix itself may be the result.
//!
//! Each of the nine is a product less a product (`x87::product_difference`), worked in order and
//! read from memory as it then is (in place, later ones see earlier results). +0xC, +0x1C, +0x2C,
//! +0x3C are not written.

use crate::math::cells::Cells;
use crate::math::x87;

/// The x87 version, for `effects::cars_particles` (unconverted). The cells are the matrix's
/// and the result's from their first dword on.
pub fn adjugate_matrix(matrix: &(impl Cells + ?Sized), out: &(impl Cells + ?Sized)) {
    use rc2k_core::scene::matrices::COFACTORS;
    let get = |o: u32| matrix.float(o as usize / 4);
    let put = |o: u32, v: f32| out.set_float(o as usize / 4, v);
    for (to, [a, b, c, d]) in COFACTORS {
        put(to, x87::product_difference(get(a), get(b), get(c), get(d)));
    }
    for o in [0x30, 0x34, 0x38] {
        put(o, x87::negated(get(o)));
    }
}
