//! The matrices an object's place in the view is made with, and the functions that fill them:
//! shared by their seven functions.
//!
//! A matrix is sixteen floats, rows 0x10 apart; a point is turned by the first three
//! rows and columns and moved by the fourth row's first three floats.
//! The two turning matrices keep their other floats (an identity's).

use crate::math::cells::Cells;
use crate::math::engine_tables::EngineTables;
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::memory::session::FloatLike;
use crate::scene::project_point::rotate_at;
use crate::scene::view_matrices::ViewMatrices;

/// An angle's sine and cosine from the game's tables `tables`: sixteen bits a turn, the low
/// four dropped.
fn sine_cosine(tables: &EngineTables, angle: u32) -> (f32, f32) {
    let place = ((angle & 0xFFFF) >> 4) as usize;
    (
        crate::math::engine_tables::sine(tables, place).get(),
        crate::math::engine_tables::cosine(tables, place).get(),
    )
}

/// The sine and cosine a turn's caller has no use for (the sources of the work words the
/// callers outside the view's own matrices never read).
pub fn unread_turn_work() -> [crate::memory::session::PlainFloat; 2] {
    [const { crate::memory::session::PlainFloat::new(0.0) }; 2]
}

/// `scene::set_turn_x`: `turn_x` the matrix, `work` its sine and cosine.
pub fn set_turn_x(
    tables: &EngineTables,
    turn_x: &[impl FloatLike; 16],
    work: &[impl FloatLike; 2],
    angle: u32,
) {
    let (sine, cosine) = sine_cosine(tables, angle);
    work[0].set(sine);
    work[1].set(cosine);
    turn_x[5].set(cosine);
    turn_x[6].set(sine);
    turn_x[9].set(-sine);
    turn_x[10].set(cosine);
}

/// `scene::set_turn_y`.
pub fn set_turn_y(
    tables: &EngineTables,
    turn_y: &[impl FloatLike; 16],
    work: &[impl FloatLike; 2],
    angle: u32,
) {
    let (sine, cosine) = sine_cosine(tables, angle);
    work[0].set(sine);
    work[1].set(cosine);
    turn_y[0].set(cosine);
    turn_y[2].set(-sine);
    turn_y[8].set(sine);
    turn_y[10].set(cosine);
}

/// `scene::set_turn_z_16`: as `set_turn_x`, about the z axis (the matrix and
/// the work of `scene::view_matrix::set_fine_turn`, whose angle is 32 bits a turn).
pub fn set_turn_z_16(
    tables: &EngineTables,
    turn_z: &[impl FloatLike; 16],
    work: &[impl FloatLike; 2],
    angle: u32,
) {
    let (sine, cosine) = sine_cosine(tables, angle);
    work[0].set(sine);
    work[1].set(cosine);
    turn_z[0].set(cosine);
    turn_z[1].set(sine);
    turn_z[4].set(-sine);
    turn_z[5].set(cosine);
}

/// `scene::set_place`, on the placed matrix.
pub fn set_place(placed: &[impl FloatLike; 16], place: [u32; 3]) {
    crate::memory::session::write_first(
        &placed[12..15],
        &place.map(|part| x87::from_int(part as i32).to_bits()),
    );
}

/// `scene::multiply_matrices`: the first three columns of `out`, column by column, each row of
/// `a` times that column of `b`; `b`'s fourth row added to the fourth. Written straight to
/// `out`, in order, so that an `out` that is one of the two behaves as written in place.
pub fn multiply_matrices(
    a: &(impl Cells + ?Sized),
    b: &(impl Cells + ?Sized),
    out: &(impl Cells + ?Sized),
) {
    bh::at_rounding!(|r| multiply_matrices_with(r, a, b, out));
}

/// `multiply_matrices` at the rounding `r`.
fn multiply_matrices_with<R: bh::Rounds>(
    r: R,
    a: &(impl Cells + ?Sized),
    b: &(impl Cells + ?Sized),
    out: &(impl Cells + ?Sized),
) {
    for column in 0..3 {
        for row in 0..4 {
            // `x87::row_times_column`: a's row times b's column, p1 + (p2 + p3).
            let p = |k: usize| r.ld(a.float(row * 4 + k)) * r.ld(b.float(k * 4 + column));
            let (p1, p2, p3) = (p(0), p(1), p(2));
            out.set_float(row * 4 + column, r.stf(p1 + (p2 + p3)));
        }
        let sum = r.ld(b.float(12 + column)) + r.ld(out.float(12 + column));
        out.set_float(12 + column, r.stf(sum));
    }
}

/// `scene::copy_matrix_with_place`: `out` = `from`, dword by dword upwards,
/// then the first three floats of `other`'s fourth row added to `out`'s.
pub fn copy_matrix_with_place(
    from: &(impl Cells + ?Sized),
    other: &(impl Cells + ?Sized),
    out: &(impl Cells + ?Sized),
) {
    crate::math::cells::copy_cells(from, out, 0x10);
    for n in 12..15 {
        let sum = bh::ld(other.float(n)) + bh::ld(out.float(n));
        out.set_float(n, bh::stf(sum));
    }
}

/// `scene::set_light_direction`: in the view's matrices `v`, turned by the view's
/// turn `view_turn` (the view's parameter, kept in the shared pieces).
pub fn set_light_direction(
    tables: &EngineTables,
    v: &ViewMatrices,
    direction: &[impl FloatLike; 3],
    view_turn: &[impl FloatLike; 16],
    light: &crate::scene::object_light::ObjectLight,
) {
    set_turn_x(
        tables,
        &v.turn_x_matrix,
        &v.turn_x_work,
        (light.light_turns[1].get() << 8).wrapping_neg(),
    );
    set_turn_y(
        tables,
        &v.turn_y_matrix,
        &v.turn_y_work,
        (light.light_turns[0].get() << 8).wrapping_neg(),
    );
    multiply_matrices(&v.turn_x_matrix, &v.turn_y_matrix, &v.turned_matrix);
    multiply_matrices(&v.turned_matrix, view_turn, &v.lit_matrix);
    // `fld1; fldz; fldz; call scene::rotate_point`, the results popped in turn.
    let ahead = rotate_at(
        [0, 0, 1],
        &v.lit_matrix
            .each_ref()
            .map(crate::memory::session::PlainFloat::get),
    );
    for (cell, value) in direction.iter().zip(ahead) {
        cell.set(value);
    }
}

/// `scene::make_object_matrix`.
pub fn make_object_matrix(
    tables: &EngineTables,
    v: &ViewMatrices,
    view_turn: &[impl FloatLike; 16],
    light: &crate::scene::object_light::ObjectLight,
    object: &crate::scene::object_place::ObjectPlace,
) {
    set_light_direction(tables, v, &light.light_direction, view_turn, light);
    set_turn_y(
        tables,
        &v.turn_y_matrix,
        &v.turn_y_work,
        object.angles[1].get().wrapping_neg(),
    );
    set_turn_x(
        tables,
        &v.turn_x_matrix,
        &v.turn_x_work,
        object.angles[0].get().wrapping_neg(),
    );
    let place: [u32; 3] = crate::memory::session::read_bits(&object.place);
    set_place(&v.placed_matrix, place);
    multiply_matrices(&v.turn_y_matrix, &v.turn_x_matrix, &v.turned_matrix);
    let size: u32 = object.size.get();
    let factor = x87::int_over_16(size as i32);
    for cell in &v.turned_matrix {
        cell.set(x87::mul(cell, factor));
    }
    copy_matrix_with_place(&v.turned_matrix, &v.placed_matrix, &v.made_object_matrix);
}
