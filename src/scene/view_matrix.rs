//! The view's matrices and the functions that make them: shared by their eight functions.
//!
//! The three fine turnings take an angle of 32 bits a turn and work the sine and
//! cosine out (`fsincos`), where `scene::set_turn_x` and `_y` look sixteen bits up in
//! tables; the first two write the same matrices as those.

use crate::math::cells::{Cells, Off, Words};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::memory::session::PlainFloat;
use crate::race::owner_course::CourseReads;
use crate::scene::matrix;
use crate::scene::matrix::Matrix;
use crate::scene::view_matrices::ViewMatrices;

/// Which of the three turnings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// The fine turn (32 bits a turn) about `axis`.
pub fn set_fine_turn(course: CourseReads<'_>, matrices: &ViewMatrices, axis: Axis, angle: u32) {
    let (sine, cosine) = x87::sine_cosine(
        angle as i32,
        course.numbers.k65536.get(),
        course.numbers.one_over_two_pi.get(),
    );
    let [sine, cosine] = [sine, cosine].map(f32::from_bits);
    let negated = -sine;
    // The work pair, then where the cosine, the sine, the sine negated and the
    // cosine go in the matrix.
    // The work pair, then where the cosine, the sine, the sine negated and the
    // cosine go in the matrix.
    let mut kept = matrices.borrow_mut();
    let kept = &mut *kept;
    let (work, rows, places): (&mut [f32; 2], &mut Matrix, [usize; 4]) = match axis {
        Axis::X => (
            &mut kept.turn_x_work,
            &mut kept.turn_x_matrix,
            [5, 6, 9, 10],
        ),
        Axis::Y => (
            &mut kept.turn_y_work,
            &mut kept.turn_y_matrix,
            [0, 8, 2, 10],
        ),
        Axis::Z => (&mut kept.turn_z_work, &mut kept.turn_z_matrix, [0, 1, 4, 5]),
    };
    *work = [sine, cosine];
    for (place, value) in places.into_iter().zip([cosine, sine, negated, cosine]) {
        rows[place] = value;
    }
}

/// `scene::unit_vector`: three floats made one long, into `out`; all 0 when their length compares
/// equal to `Numbers.zero` (0, or no number). Minus zero made zero. Returns the length.
pub fn unit_vector(
    course: CourseReads<'_>,
    from: &(impl Cells + ?Sized),
    out: &(impl Cells + ?Sized),
) -> u32 {
    // `x87::length`: sqrt(x x + (y y + z z)).
    let (x, y, z) = (
        bh::ld(from.float(0)),
        bh::ld(from.float(1)),
        bh::ld(from.float(2)),
    );
    let length = bh::st((x * x + (y * y + z * z)).sqrt());
    if x87::equal_or_unordered(length, course.numbers.zero.get()) {
        for n in 0..3 {
            out.set_float(n, 0.0);
        }
        return length;
    }
    for n in 0..3 {
        out.set_float(n, x87::div(from.float(n), length));
    }
    for n in 0..3 {
        let v = out.float(n);
        if v == 0.0 && v.is_sign_negative() {
            out.set_float(n, 0.0);
        }
    }
    length
}

/// `scene::cross_product`: `out` = `b` x `c`, written part
/// by part.
pub fn cross_product(
    b: &(impl Cells + ?Sized),
    c: &(impl Cells + ?Sized),
    out: &(impl Cells + ?Sized),
) {
    let [x, y, z] = cross_of(b, c);
    out.set_float(0, x);
    out.set_float(1, y);
    out.set_float(2, z);
}

/// `b` x `c` as three floats (the part `cross_product` writes).
pub fn cross_of(b: &(impl Words + ?Sized), c: &(impl Words + ?Sized)) -> [f32; 3] {
    let part = |x: usize, y: usize| {
        x87::product_difference(c.float(x), b.float(y), b.float(x), c.float(y))
    };
    [part(2, 1), part(0, 2), part(1, 0)]
}

/// `scene::square_matrix`: a turning's three rows made one long and at right angles again, from its
/// third row and its second. Returns the second `unit_vector`'s length. The work is a local, so
/// that views drawn beside each other do not work in each other's rows.
pub fn square_matrix(course: CourseReads<'_>, at: &(impl Cells + ?Sized)) -> u32 {
    let squared = [const { PlainFloat::new(0.0) }; 0x10];
    let (first, second, third) = (Off(&squared, 0), Off(&squared, 4), Off(&squared, 8));
    unit_vector(course, &Off(at, 8), &third);
    cross_product(&Off(at, 4), &third, &first);
    let length = unit_vector(course, &first, &first);
    cross_product(&third, &first, &second);
    crate::math::cells::copy_cells(&squared, at, 0x10);
    length
}

/// `scene::invert_turn`: the turning at `from` undone, into `out`: its nine
/// floats' cofactors laid out the other way round, then squared. Returns `square_matrix`'s length.
pub fn invert_turn(
    course: CourseReads<'_>,
    from: &(impl Cells + ?Sized),
    out: &(impl Cells + ?Sized),
) -> u32 {
    let mut m = [0f32; 9];
    for (n, value) in m.iter_mut().enumerate() {
        *value = x87::loaded(from.float((n / 3) * 4 + n % 3));
    }
    let [a, b, c, d, e, f, g, h, i] = m;
    let plain = x87::product_difference;
    let negated = |a, b, c, d| x87::negated(plain(a, b, c, d));
    let made = [
        plain(e, i, f, h),
        negated(b, i, c, h),
        plain(b, f, c, e),
        negated(d, i, f, g),
        plain(a, i, c, g),
        negated(a, f, c, d),
        plain(d, h, e, g),
        negated(a, h, b, g),
        plain(a, e, b, d),
    ];
    for (n, value) in made.into_iter().enumerate() {
        out.set_float((n / 3) * 4 + n % 3, x87::loaded(value));
    }
    square_matrix(course, out)
}

/// `invert_turn` on matrices as values: the turning of `from` undone (every float of the result
/// is set, so the result does not depend on what the matrix it replaces held).
pub fn inverted_turn(course: CourseReads<'_>, from: &matrix::Matrix) -> matrix::Matrix {
    let from: [PlainFloat; 16] = from.map(PlainFloat::new);
    let out = [const { PlainFloat::new(0.0) }; 16];
    invert_turn(course, &from, &out);
    matrix::matrix_of(&out)
}

/// `scene::set_view_matrices`, working in `scratch` (the front end's).
pub fn set_view_matrices(
    course: CourseReads<'_>,
    frame: &crate::race::owner_frame::Frame,
    scratch: &ViewMatrices,
) {
    let old_view_matrix = frame.view_kept.borrow().view_matrix;
    let [a0, a1, a2, ..] = frame.view_and_records.borrow().view_vars.view_angles;
    let angles = [a0, a1, a2];
    set_fine_turn(course, scratch, Axis::Y, angles[0].wrapping_neg());
    set_fine_turn(course, scratch, Axis::X, angles[1].wrapping_neg());
    set_fine_turn(course, scratch, Axis::Z, angles[2].wrapping_neg());
    let place = frame
        .view_and_records
        .borrow()
        .view_vars
        .view_place
        .map(u32::wrapping_neg);
    let (view_turn, view_place_matrix, view_matrix) = {
        let mut kept = scratch.borrow_mut();
        let v = &mut *kept;
        matrix::multiply_matrices(&v.turn_y_matrix, &v.turn_x_matrix, &mut v.lit_matrix);
        // The turn starts as the identity, its fourth column kept by the product.
        let view_turn = matrix::product(
            &v.lit_matrix,
            &v.turn_z_matrix,
            crate::race::matrices_store::identity_matrix(),
        );
        matrix::set_place(&mut v.placed_matrix, place);
        let view_place_matrix = v.placed_matrix;
        let view_matrix = matrix::product(&view_place_matrix, &view_turn, old_view_matrix);
        (view_turn, view_place_matrix, view_matrix)
    };
    let view_back_matrix = inverted_turn(course, &view_turn);
    {
        let mut kept = frame.view_kept.borrow_mut();
        kept.view_turn_matrix = view_turn;
        kept.view_place_matrix = view_place_matrix;
        kept.view_matrix = view_matrix;
        kept.view_back_matrix = view_back_matrix;
    }
    // The front end's own (its drawing reads them).
    let mut kept = scratch.borrow_mut();
    kept.view_turn_matrix = view_turn;
    kept.view_place_matrix = view_place_matrix;
    kept.view_matrix = view_matrix;
    kept.view_back_matrix = view_back_matrix;
}

// ---- the converted versions (`rc2k_core::scene::matrices`) ----
