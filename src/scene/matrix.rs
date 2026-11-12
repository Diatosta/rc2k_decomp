//! The matrices an object's place in the view is made with, and the functions that fill them:
//! shared by their seven functions.
//!
//! A matrix is sixteen floats, rows 0x10 apart; a point is turned by the first three
//! rows and columns and moved by the fourth row's first three floats.
//! The two turning matrices keep their other floats (an identity's) from the image.

use crate::math::cells::Cells;
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::scene::engine_tables_store::EngineTables;
use crate::scene::project_point::rotate_at;
use crate::scene::view_matrices::ViewMatrices;
use crate::session::Float;

/// An angle's sine and cosine from the game's tables `tables`: sixteen bits a turn, the low
/// four dropped.
fn sine_cosine(tables: &EngineTables, angle: u32) -> (f32, f32) {
    let place = ((angle & 0xFFFF) >> 4) as usize;
    (
        crate::scene::engine_tables_store::sine(tables, place).get(),
        crate::scene::engine_tables_store::cosine(tables, place).get(),
    )
}

/// `scene::set_turn_x` (0x426FE1): `turn_x` the matrix, `work` its sine and cosine.
pub fn set_turn_x(tables: &EngineTables, turn_x: &[Float; 16], work: &[Float; 2], angle: u32) {
    let (sine, cosine) = sine_cosine(tables, angle);
    work[0].set(sine);
    work[1].set(cosine);
    turn_x[5].set(cosine);
    turn_x[6].set(sine);
    turn_x[9].set(-sine);
    turn_x[10].set(cosine);
}

/// `scene::set_turn_y` (0x42702A).
pub fn set_turn_y(tables: &EngineTables, turn_y: &[Float; 16], work: &[Float; 2], angle: u32) {
    let (sine, cosine) = sine_cosine(tables, angle);
    work[0].set(sine);
    work[1].set(cosine);
    turn_y[0].set(cosine);
    turn_y[2].set(-sine);
    turn_y[8].set(sine);
    turn_y[10].set(cosine);
}

/// `scene::set_turn_z_16` (0x427072): as `set_turn_x`, about the z axis (the matrix and
/// the work of `scene::set_turn_z`, whose angle is 32 bits a turn).
pub fn set_turn_z_16(tables: &EngineTables, turn_z: &[Float; 16], work: &[Float; 2], angle: u32) {
    let (sine, cosine) = sine_cosine(tables, angle);
    work[0].set(sine);
    work[1].set(cosine);
    turn_z[0].set(cosine);
    turn_z[1].set(sine);
    turn_z[4].set(-sine);
    turn_z[5].set(cosine);
}

/// `scene::set_place` (0x4271AE), on the placed matrix. (The original also left the place's z
/// in `SceneDraw.scratch`; the drawing never read it, `physics::car_collisions` stores it there
/// itself.)
pub fn set_place(placed: &[Float; 16], place: [u32; 3]) {
    crate::session::write_first(
        &placed[12..15],
        &place.map(|part| x87::from_int(part as i32).to_bits()),
    );
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
            let p = |k: usize| bh::ld(a.float(row * 4 + k)) * bh::ld(b.float(k * 4 + column));
            let (p1, p2, p3) = (p(0), p(1), p(2));
            out.set_float(row * 4 + column, bh::stf(p1 + (p2 + p3)));
        }
        let sum = bh::ld(b.float(12 + column)) + bh::ld(out.float(12 + column));
        out.set_float(12 + column, bh::stf(sum));
    }
}

/// `scene::copy_matrix_with_place` (0x4271E4): `out` = `from`, dword by dword upwards,
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

/// `scene::set_light_direction` (0x427326): in the view's matrices `v`, turned by the view's
/// turn `view_turn` (the view's parameter, kept in the shared pieces).
pub fn set_light_direction(
    tables: &EngineTables,
    v: &ViewMatrices,
    direction: &[Float; 3],
    view_turn: &[Float; 16],
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
    let ahead = rotate_at([0, 0, 1], &v.lit_matrix.each_ref().map(Float::get));
    for (cell, value) in direction.iter().zip(ahead) {
        cell.set(value);
    }
}

/// `scene::make_object_matrix` (0x427812).
pub fn make_object_matrix(
    tables: &EngineTables,
    v: &ViewMatrices,
    view_turn: &[Float; 16],
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
    let place: [u32; 3] = crate::session::read_bits(&object.place);
    set_place(&v.placed_matrix, place);
    multiply_matrices(&v.turn_y_matrix, &v.turn_x_matrix, &v.turned_matrix);
    let size: u32 = object.size.get();
    let factor = x87::int_over_16(size as i32);
    for cell in &v.turned_matrix {
        cell.set(x87::mul(cell, factor));
    }
    copy_matrix_with_place(&v.turned_matrix, &v.placed_matrix, &v.made_object_matrix);
}
