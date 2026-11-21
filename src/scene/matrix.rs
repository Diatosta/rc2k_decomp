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

/// A 4x4 matrix as sixteen floats, rows four apart.
pub type Matrix = [f32; 16];

/// Zeros with the floats `ones` set to one: what a turn matrix or the placed matrix is before its
/// turn or place is set (a turn sets four floats, a place three, the rest stay).
const fn ones_at(ones: &[usize]) -> Matrix {
    let mut matrix = [0.0; 16];
    let mut k = 0;
    while k < ones.len() {
        matrix[ones[k]] = 1.0;
        k += 1;
    }
    matrix
}

/// The turn about x before its angle is set: `turned_x`'s base.
pub const TURN_X_BASE: Matrix = ones_at(&[0, 15]);
/// The turn about y before its angle is set: `turned_y`'s base.
pub const TURN_Y_BASE: Matrix = ones_at(&[5, 15]);
/// The turn about z before its angle is set: `turned_z_16`'s base.
pub const TURN_Z_BASE: Matrix = ones_at(&[10, 15]);
/// The placed matrix before its place is set: `placed`'s base.
pub const PLACED_BASE: Matrix = ones_at(&[0, 5, 10, 15]);

/// The sixteen floats `cells` hold.
pub fn matrix_of(cells: &(impl Cells + ?Sized)) -> Matrix {
    std::array::from_fn(|n| cells.float(n))
}

/// `matrix` into the sixteen cells `cells`.
pub fn store_matrix(cells: &(impl Cells + ?Sized), matrix: &Matrix) {
    for (n, &value) in matrix.iter().enumerate() {
        cells.set_float(n, value);
    }
}

/// `matrix` turned about x by `angle` (`scene::set_turn_x`): its fifth, sixth, tenth and eleventh
/// floats set; with the sine and the cosine.
pub fn turned_x(tables: &EngineTables, mut matrix: Matrix, angle: u32) -> (Matrix, [f32; 2]) {
    let (sine, cosine) = sine_cosine(tables, angle);
    matrix[5] = cosine;
    matrix[6] = sine;
    matrix[9] = -sine;
    matrix[10] = cosine;
    (matrix, [sine, cosine])
}

/// `matrix` turned about y by `angle` (`scene::set_turn_y`), with the sine and the cosine.
pub fn turned_y(tables: &EngineTables, mut matrix: Matrix, angle: u32) -> (Matrix, [f32; 2]) {
    let (sine, cosine) = sine_cosine(tables, angle);
    matrix[0] = cosine;
    matrix[2] = -sine;
    matrix[8] = sine;
    matrix[10] = cosine;
    (matrix, [sine, cosine])
}

/// `matrix` turned about z by `angle`, sixteen bits a turn (`scene::set_turn_z_16`), with the
/// sine and the cosine.
pub fn turned_z_16(tables: &EngineTables, mut matrix: Matrix, angle: u32) -> (Matrix, [f32; 2]) {
    let (sine, cosine) = sine_cosine(tables, angle);
    matrix[0] = cosine;
    matrix[1] = sine;
    matrix[4] = -sine;
    matrix[5] = cosine;
    (matrix, [sine, cosine])
}

/// `matrix` with the place `place` in its fourth row (`scene::set_place`).
pub fn placed(mut matrix: Matrix, place: [u32; 3]) -> Matrix {
    for (n, part) in place.into_iter().enumerate() {
        matrix[12 + n] = f32::from_bits(x87::from_int(part as i32).to_bits());
    }
    matrix
}

/// `scene::multiply_matrices`: `out` with its first three columns replaced, column by column,
/// each row of `a` times that column of `b`, `b`'s fourth row added to the fourth. `out` is none of
/// `a` and `b` (the cells' version, `multiply_matrices`, works on one that is).
pub fn product(a: &Matrix, b: &Matrix, mut out: Matrix) -> Matrix {
    bh::at_rounding!(|r| product_with(r, a, b, &mut out));
    out
}

/// `product` at the rounding `r`.
fn product_with<R: bh::Rounds>(r: R, a: &Matrix, b: &Matrix, out: &mut Matrix) {
    for column in 0..3 {
        for row in 0..4 {
            // `x87::row_times_column`: a's row times b's column, p1 + (p2 + p3).
            let p = |k: usize| r.ld(a[row * 4 + k]) * r.ld(b[k * 4 + column]);
            let (p1, p2, p3) = (p(0), p(1), p(2));
            out[row * 4 + column] = r.stf(p1 + (p2 + p3));
        }
        let sum = r.ld(b[12 + column]) + r.ld(out[12 + column]);
        out[12 + column] = r.stf(sum);
    }
}

/// `scene::copy_matrix_with_place`: `from`, the first three floats of `other`'s fourth row added
/// to its own (`other` is not the matrix the result goes to).
pub fn copied_with_place(from: &Matrix, other: &Matrix) -> Matrix {
    let mut out = *from;
    for n in 12..15 {
        let sum = bh::ld(other[n]) + bh::ld(out[n]);
        out[n] = bh::stf(sum);
    }
    out
}

/// `matrix` with every float times `factor`.
pub fn scaled(mut matrix: Matrix, factor: f32) -> Matrix {
    for value in &mut matrix {
        *value = x87::mul(*value, factor);
    }
    matrix
}

/// `scene::set_turn_x`: `turn_x` the matrix, `work` its sine and cosine.
pub fn set_turn_x(
    tables: &EngineTables,
    turn_x: &[impl FloatLike; 16],
    work: &[impl FloatLike; 2],
    angle: u32,
) {
    let (matrix, [sine, cosine]) = turned_x(tables, matrix_of(turn_x), angle);
    work[0].set(sine);
    work[1].set(cosine);
    store_matrix(turn_x, &matrix);
}

/// `scene::set_turn_y`.
pub fn set_turn_y(
    tables: &EngineTables,
    turn_y: &[impl FloatLike; 16],
    work: &[impl FloatLike; 2],
    angle: u32,
) {
    let (matrix, [sine, cosine]) = turned_y(tables, matrix_of(turn_y), angle);
    work[0].set(sine);
    work[1].set(cosine);
    store_matrix(turn_y, &matrix);
}

/// `scene::set_turn_z_16`: as `set_turn_x`, about the z axis (the matrix and
/// the work of `scene::view_matrix::set_fine_turn`, whose angle is 32 bits a turn).
pub fn set_turn_z_16(
    tables: &EngineTables,
    turn_z: &[impl FloatLike; 16],
    work: &[impl FloatLike; 2],
    angle: u32,
) {
    let (matrix, [sine, cosine]) = turned_z_16(tables, matrix_of(turn_z), angle);
    work[0].set(sine);
    work[1].set(cosine);
    store_matrix(turn_z, &matrix);
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
    store_matrix(out, &copied_with_place(&matrix_of(from), &matrix_of(other)));
}

/// `scene::set_light_direction`: in the view's matrices `v`, turned by the view's
/// turn `view_turn` (the view's parameter, kept in the shared pieces).
pub fn set_light_direction(
    tables: &EngineTables,
    v: &ViewMatrices,
    view_turn: &[impl FloatLike; 16],
    light: &crate::scene::object_light::ObjectLight,
) {
    let turns = light.borrow().light_turns;
    set_turn_x(
        tables,
        &v.turn_x_matrix,
        &v.turn_x_work,
        (turns[1] << 8).wrapping_neg(),
    );
    set_turn_y(
        tables,
        &v.turn_y_matrix,
        &v.turn_y_work,
        (turns[0] << 8).wrapping_neg(),
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
    light.borrow_mut().light_direction = ahead.map(f32::to_bits);
}

/// `scene::make_object_matrix`.
pub fn make_object_matrix(
    tables: &EngineTables,
    v: &ViewMatrices,
    view_turn: &[impl FloatLike; 16],
    light: &crate::scene::object_light::ObjectLight,
    object: &crate::scene::object_place::ObjectPlace,
) {
    set_light_direction(tables, v, view_turn, light);
    set_turn_y(
        tables,
        &v.turn_y_matrix,
        &v.turn_y_work,
        object.borrow().angles[1].wrapping_neg(),
    );
    set_turn_x(
        tables,
        &v.turn_x_matrix,
        &v.turn_x_work,
        object.borrow().angles[0].wrapping_neg(),
    );
    let place: [u32; 3] = object.borrow().place;
    set_place(&v.placed_matrix, place);
    multiply_matrices(&v.turn_y_matrix, &v.turn_x_matrix, &v.turned_matrix);
    let size: u32 = object.borrow().size;
    let factor = x87::int_over_16(size as i32);
    for cell in &v.turned_matrix {
        cell.set(x87::mul(cell, factor));
    }
    copy_matrix_with_place(&v.turned_matrix, &v.placed_matrix, &v.made_object_matrix);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::session::PlainFloat;

    /// Sixteen floats from a seed, some negative and fractional.
    fn matrix_from(seed: u32) -> Matrix {
        std::array::from_fn(|n| {
            let k = seed
                .wrapping_mul(2_654_435_761)
                .wrapping_add(n as u32 * 40_503);
            (k >> 8) as f32 / 65_536.0 - 100.0
        })
    }

    #[test]
    fn the_value_product_matches_the_cells_product() {
        for seed in 0..64 {
            let (a, b, out) = (
                matrix_from(seed),
                matrix_from(seed + 100),
                matrix_from(seed + 200),
            );
            let cells = PlainFloat::array(out);
            multiply_matrices(&PlainFloat::array(a), &PlainFloat::array(b), &cells);
            let expected: Vec<u32> = cells.iter().map(|c| c.get().to_bits()).collect();
            let got: Vec<u32> = product(&a, &b, out).iter().map(|v| v.to_bits()).collect();
            assert_eq!(got, expected, "seed {seed}");
        }
    }

    #[test]
    fn a_copy_with_place_adds_the_other_matrixs_place_to_the_copy() {
        for seed in 0..64 {
            let (from, other, out) = (
                matrix_from(seed),
                matrix_from(seed + 100),
                matrix_from(seed + 200),
            );
            let cells = PlainFloat::array(out);
            copy_matrix_with_place(&PlainFloat::array(from), &PlainFloat::array(other), &cells);
            let mut expected = from;
            for n in 12..15 {
                expected[n] = (f64::from(other[n]) + f64::from(from[n])) as f32;
            }
            assert_eq!(
                matrix_of(&cells).map(f32::to_bits),
                expected.map(f32::to_bits)
            );
            assert_eq!(copied_with_place(&from, &other), expected);
        }
    }

    #[test]
    fn a_place_sets_the_fourth_row_and_keeps_the_rest() {
        let base = matrix_from(7);
        let at = placed(base, [3, 0xFFFF_FFFF, 0x1_0000]);
        assert_eq!(at[..12], base[..12]);
        assert_eq!(at[15].to_bits(), base[15].to_bits());
        assert_eq!(at[12], 3.0);
        assert_eq!(at[13], -1.0);
        assert_eq!(at[14], 65_536.0);
    }
}
