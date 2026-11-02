//! The matrices an object's place in the view is made with, and the functions that fill them:
//! shared by their seven functions.
//!
//! A matrix is sixteen floats, rows 0x10 apart; a point is turned by the first three
//! rows and columns and moved by the fourth row's first three floats.
//! The two turning matrices keep their other floats (an identity's) from the image.

use crate::math::cells::Cells;
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::matrices_store::MATRICES;
use crate::scene::draw_store::DRAW;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::project_point::rotate;

/// An angle's sine and cosine from the game's tables: sixteen bits a turn, the low
/// four dropped.
fn sine_cosine(angle: u32) -> (u32, u32) {
    let place = ((angle & 0xFFFF) >> 4) as usize;
    {
        (
            crate::scene::engine_tables_store::sine(place).bits(),
            crate::scene::engine_tables_store::cosine(place).bits(),
        )
    }
}

/// `scene::set_turn_x` (0x426FE1).
pub fn set_turn_x(angle: u32) {
    let (sine, cosine) = sine_cosine(angle);
    crate::session::write_bits(&DRAW.turn_x_work, [sine, cosine]);
    MATRICES.turn_x_matrix[5].set_bits(cosine);
    MATRICES.turn_x_matrix[6].set_bits(sine);
    MATRICES.turn_x_matrix[9].set_bits(sine ^ 0x8000_0000);
    MATRICES.turn_x_matrix[10].set_bits(cosine);
}

/// `scene::set_turn_y` (0x42702A).
pub fn set_turn_y(angle: u32) {
    let (sine, cosine) = sine_cosine(angle);
    crate::session::write_bits(&DRAW.turn_y_work, [sine, cosine]);
    MATRICES.turn_y_matrix[0].set_bits(cosine);
    MATRICES.turn_y_matrix[2].set_bits(sine ^ 0x8000_0000);
    MATRICES.turn_y_matrix[8].set_bits(sine);
    MATRICES.turn_y_matrix[10].set_bits(cosine);
}

/// `scene::set_turn_z_16` (0x427072): as `set_turn_x`, about the z axis (the matrix and
/// the work of `scene::set_turn_z`, whose angle is 32 bits a turn).
pub fn set_turn_z_16(angle: u32) {
    let (sine, cosine) = sine_cosine(angle);
    crate::session::write_bits(&DRAW.turn_z_work, [sine, cosine]);
    crate::session::write_first(&MATRICES.turn_z_matrix, &[cosine, sine]);
    MATRICES.turn_z_matrix[4].set_bits(sine ^ 0x8000_0000);
    MATRICES.turn_z_matrix[5].set_bits(cosine);
}

/// `scene::set_place` (0x4271AE).
pub fn set_place(place: [u32; 3]) {
    crate::session::write_first(
        &MATRICES.placed_matrix[12..15],
        &place.map(|part| x87::from_int(part as i32)),
    );
    DRAW.scratch.set_bits(place[2]);
}

/// `scene::multiply_matrices` (0x42720D): the first three columns of `out`, column by
/// column, each row of `a` times that column of `b`; `b`'s fourth row added to the
/// fourth. Written straight to `out`, in the original's order, so that an `out` that
/// is one of the two behaves as there.
pub fn multiply_matrices(
    a: &(impl Cells + ?Sized),
    b: &(impl Cells + ?Sized),
    out: &(impl Cells + ?Sized),
) {
    for column in 0..3 {
        for row in 0..4 {
            // `x87::row_times_column`: a's row times b's column, p1 + (p2 + p3).
            let p = |k: usize| bh::ld(a.bits(row * 4 + k)) * bh::ld(b.bits(k * 4 + column));
            let (p1, p2, p3) = (p(0), p(1), p(2));
            out.set_bits(row * 4 + column, bh::st(p1 + (p2 + p3)));
        }
        let sum = bh::ld(b.bits(12 + column)) + bh::ld(out.bits(12 + column));
        out.set_bits(12 + column, bh::st(sum));
    }
}

/// `scene::copy_matrix_with_place` (0x4271E4): `out` = `from`, dword by dword upwards,
/// then the first three floats of `other`'s fourth row added to `out`'s.
pub fn copy_matrix_with_place(
    from: &(impl Cells + ?Sized),
    other: &(impl Cells + ?Sized),
    out: &(impl Cells + ?Sized),
) {
    for n in 0..0x10 {
        out.set_bits(n, from.bits(n));
    }
    for n in 12..15 {
        let sum = bh::ld(other.bits(n)) + bh::ld(out.bits(n));
        out.set_bits(n, bh::st(sum));
    }
}

/// `scene::set_light_direction` (0x427326).
pub fn set_light_direction() {
    set_turn_x((OBJECT_WORK.light_turns[1].bits() << 8).wrapping_neg());
    set_turn_y((OBJECT_WORK.light_turns[0].bits() << 8).wrapping_neg());
    multiply_matrices(
        &MATRICES.turn_x_matrix,
        &MATRICES.turn_y_matrix,
        &MATRICES.turned_matrix,
    );
    multiply_matrices(
        &MATRICES.turned_matrix,
        &MATRICES.view_turn_matrix,
        &MATRICES.lit_matrix,
    );
    // `fld1; fldz; fldz; call scene::rotate_point`, the results popped in turn.
    let ahead = [0, 0, 1];
    let mut record = [0u32; 7];
    rotate(
        &ahead,
        &MATRICES
            .lit_matrix
            .each_ref()
            .map(super::super::session::Float::bits),
        &mut record,
    );
    for (cell, &bits) in DRAW.light_direction.iter().zip(&record[4..7]) {
        cell.set_bits(bits);
    }
}

/// `scene::make_object_matrix` (0x427812).
pub fn make_object_matrix() {
    set_light_direction();
    set_turn_y(OBJECT_WORK.angles[1].bits().wrapping_neg());
    set_turn_x(OBJECT_WORK.angles[0].bits().wrapping_neg());
    let place: [u32; 3] = crate::session::read_bits(&OBJECT_WORK.place);
    set_place(place);
    multiply_matrices(
        &MATRICES.turn_y_matrix,
        &MATRICES.turn_x_matrix,
        &MATRICES.turned_matrix,
    );
    let size: u32 = OBJECT_WORK.size.bits();
    let factor = x87::int_over_16(size as i32);
    DRAW.scratch.set_bits(factor);
    for n in 0..0x10 {
        let cell = &MATRICES.turned_matrix[n];
        cell.set_bits(x87::mul(cell.bits(), factor));
    }
    copy_matrix_with_place(
        &MATRICES.turned_matrix,
        &MATRICES.placed_matrix,
        &MATRICES.made_object_matrix,
    );
}
