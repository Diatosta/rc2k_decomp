//! `scene::adjugate_matrix` (0x4BCF6B): a matrix's turn undone without the division: its
//! 3 x 3 part's cofactors, transposed (rows 0x10 bytes apart), and its place (+0x30 ..
//! +0x38) negated, not turned.
//!
//! In:  esi the matrix, edi where the result goes (the matrix itself allowed)
//! Out: every register kept
//!
//! Each of the nine a product less a product (`x87::product_difference`), written in the
//! original's order and read from memory as it then is (in place, later ones see earlier
//! results). +0xC, +0x1C, +0x2C, +0x3C are not written. One caller, `sub_4C9AF6`. Refuses a
//! null matrix or result. The Rust version is `adjugate_matrix_plain`; the x87's stays for
//! `marks::cars_particles`.

use crate::math::x87;

/// The Rust version (see the header), the x87's: for `marks::cars_particles`
/// (unconverted).
pub unsafe fn adjugate_matrix(matrix: u32, out: u32) {
    use rc2k_core::scene::matrices::COFACTORS;
    let get = |o: u32| {
        crate::pointers::read_or_end::<u32>(matrix + o, "scene::adjugate_matrix's operand")
    };
    let put = |o: u32, v: u32| {
        crate::pointers::write_or_end::<u32>(out + o, v, "scene::adjugate_matrix's operand")
    };
    for (to, [a, b, c, d]) in COFACTORS {
        put(to, x87::product_difference(get(a), get(b), get(c), get(d)));
    }
    for o in [0x30, 0x34, 0x38] {
        put(o, x87::negated(get(o)));
    }
}
