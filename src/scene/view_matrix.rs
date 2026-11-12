//! The view's matrices and the functions that make them: shared by their eight functions.
//!
//! The three fine turnings take an angle of 32 bits a turn and work the sine and
//! cosine out (`fsincos`), where `scene::set_turn_x` and `_y` look sixteen bits up in
//! tables; the first two write the same matrices as those.

use crate::math::cells::{Cells, Off};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::owner_course::Course;
use crate::scene::matrix;
use crate::scene::view_matrices::ViewMatrices;
use crate::session::Float;

/// Which of the three turnings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// `scene::set_fine_turn_x` (0x4270BA), `scene::set_fine_turn_y` (0x42710C),
/// `scene::set_turn_z` (0x42715D).
pub fn set_fine_turn(course: &Course, matrices: &ViewMatrices, axis: Axis, angle: u32) {
    let (sine, cosine) = x87::sine_cosine(
        angle as i32,
        course.numbers.k65536.get(),
        course.numbers.one_over_two_pi.get(),
    );
    let [sine, cosine] = [sine, cosine].map(f32::from_bits);
    let negated = -sine;
    // The work pair, then where the cosine, the sine, the sine negated and the
    // cosine go in the matrix.
    let (work, rows, places): (&[Float; 2], &[Float; 16], [usize; 4]) = match axis {
        Axis::X => (
            &matrices.turn_x_work,
            &matrices.turn_x_matrix,
            [5, 6, 9, 10],
        ),
        Axis::Y => (
            &matrices.turn_y_work,
            &matrices.turn_y_matrix,
            [0, 8, 2, 10],
        ),
        Axis::Z => (&matrices.turn_z_work, &matrices.turn_z_matrix, [0, 1, 4, 5]),
    };
    work[0].set(sine);
    work[1].set(cosine);
    for (place, value) in places.into_iter().zip([cosine, sine, negated, cosine]) {
        rows[place].set(value);
    }
}

/// `scene::unit_vector` (0x420EF8): three floats made one long, into `out`; all 0 when
/// their length compares equal to `Numbers.zero` (0, or no number). Minus zero made zero.
/// Returns the length (the original left it in `SceneDraw.scratch`: the physics' callers store it
/// there, the drawing's never read it).
pub fn unit_vector(
    course: &Course,
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

/// `scene::cross_product` (0x420F8D): `out` = `b` x `c`, written part
/// by part.
pub fn cross_product(
    b: &(impl Cells + ?Sized),
    c: &(impl Cells + ?Sized),
    out: &(impl Cells + ?Sized),
) {
    let part = |x: usize, y: usize| {
        x87::product_difference(c.float(x), b.float(y), b.float(x), c.float(y))
    };
    out.set_float(0, part(2, 1));
    out.set_float(1, part(0, 2));
    out.set_float(2, part(1, 0));
}

/// `scene::square_matrix` (0x420E94): a turning's three rows made one long and at right
/// angles again, from its third row and its second. Returns the second `unit_vector`'s length.
/// The original worked in `CourseLists.squared_row_0[0]` .. (16 floats, the last four never made: zeros, as the
/// image has them, copied back); here a local, so that views drawn beside each other do not
/// work in each other's rows.
pub fn square_matrix(course: &Course, at: &(impl Cells + ?Sized)) -> u32 {
    let squared = [const { Float::new(0.0) }; 0x10];
    let (first, second, third) = (Off(&squared, 0), Off(&squared, 4), Off(&squared, 8));
    unit_vector(course, &Off(at, 8), &third);
    cross_product(&Off(at, 4), &third, &first);
    let length = unit_vector(course, &first, &first);
    cross_product(&third, &first, &second);
    crate::math::cells::copy_cells(&squared, at, 0x10);
    length
}

/// `scene::invert_turn` (0x420CC2): the turning at `from` undone, into `out`: its nine
/// floats' cofactors laid out the other way round, then squared. Returns `square_matrix`'s length.
pub fn invert_turn(
    course: &Course,
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

/// `scene::set_view_matrices` (0x438FBF), working in `scratch` (the front end's).
pub fn set_view_matrices(
    course: &Course,
    frame: &crate::race::owner_frame::Frame,
    scratch: &ViewMatrices,
) {
    crate::session::write_bits(
        &frame.matrices.view_turn_matrix,
        crate::session::read_bits(&frame.matrices.identity_matrix),
    );
    let angles: [u32; 3] =
        crate::session::read_first(&frame.view_and_records.get().view_vars.view_angles);
    set_fine_turn(course, scratch, Axis::Y, angles[0].wrapping_neg());
    set_fine_turn(course, scratch, Axis::X, angles[1].wrapping_neg());
    set_fine_turn(course, scratch, Axis::Z, angles[2].wrapping_neg());
    matrix::multiply_matrices(
        &scratch.turn_y_matrix,
        &scratch.turn_x_matrix,
        &scratch.lit_matrix,
    );
    matrix::multiply_matrices(
        &scratch.lit_matrix,
        &scratch.turn_z_matrix,
        &frame.matrices.view_turn_matrix,
    );
    let place =
        crate::session::read_bits::<_, 3>(&frame.view_and_records.get().view_vars.view_place)
            .map(u32::wrapping_neg);
    matrix::set_place(&scratch.placed_matrix, place);
    crate::session::write_bits(
        &frame.matrices.view_place_matrix,
        crate::session::read_bits(&scratch.placed_matrix),
    );
    matrix::multiply_matrices(
        &scratch.placed_matrix,
        &frame.matrices.view_turn_matrix,
        &frame.matrices.view_matrix,
    );
    invert_turn(
        course,
        &frame.matrices.view_turn_matrix,
        &frame.matrices.view_back_matrix,
    );
    // The front end's own (its drawing reads them).
    let m = &frame.matrices;
    for (own, shared) in [
        (&scratch.view_turn_matrix, &m.view_turn_matrix),
        (&scratch.view_place_matrix, &m.view_place_matrix),
        (&scratch.view_matrix, &m.view_matrix),
        (&scratch.view_back_matrix, &m.view_back_matrix),
    ] {
        crate::math::cells::copy_cells(shared, own, 16);
    }
}

// ---- the converted versions (`rc2k_core::scene::matrices`) ----
