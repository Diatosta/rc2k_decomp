//! The view's matrices and the functions that make them: shared by their eight functions.
//!
//! The three fine turnings take an angle of 32 bits a turn and work the sine and
//! cosine out (`fsincos`), where `scene::set_turn_x` and `_y` look sixteen bits up in
//! tables; the first two write the same matrices as those.

use crate::math::cells::{Cells, Off};
use crate::math::x87;
use crate::math::x87_float as bh;
use crate::race::matrices_store::SceneMatrices;
use crate::race::owner_course::Course;
use crate::scene::matrix;
use crate::scene::owner::scene;
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
pub fn set_fine_turn(course: &Course, matrices: &SceneMatrices, axis: Axis, angle: u32) {
    let drawing = &scene().drawing;

    drawing.scene.scratch.set_bits(angle);
    let (sine, cosine) = x87::sine_cosine(
        angle as i32,
        course.numbers.k65536.bits(),
        course.numbers.one_over_two_pi.bits(),
    );
    let negated = sine ^ 0x8000_0000;
    // The work pair, then where the cosine, the sine, the sine negated and the
    // cosine go in the matrix.
    let (work, rows, places): (&[Float; 2], &[Float; 16], [usize; 4]) = match axis {
        Axis::X => (
            &drawing.scene.turn_x_work,
            &matrices.turn_x_matrix,
            [5, 6, 9, 10],
        ),
        Axis::Y => (
            &drawing.scene.turn_y_work,
            &matrices.turn_y_matrix,
            [0, 8, 2, 10],
        ),
        Axis::Z => (
            &drawing.scene.turn_z_work,
            &matrices.turn_z_matrix,
            [0, 1, 4, 5],
        ),
    };
    crate::session::write_bits(work, [sine, cosine]);
    for (place, value) in places.into_iter().zip([cosine, sine, negated, cosine]) {
        rows[place].set_bits(value);
    }
}

/// `scene::unit_vector` (0x420EF8): three floats made one long, into `out`; all 0 when
/// their length compares equal to `flt_5C5D90` (0, or no number). Minus zero made zero.
/// The length is left in `dword_62400C`.
pub fn unit_vector(course: &Course, from: &(impl Cells + ?Sized), out: &(impl Cells + ?Sized)) {
    // `x87::length`: sqrt(x x + (y y + z z)).
    let (x, y, z) = (
        bh::ld(from.bits(0)),
        bh::ld(from.bits(1)),
        bh::ld(from.bits(2)),
    );
    let length = bh::st((x * x + (y * y + z * z)).sqrt());
    scene().drawing.scene.scratch.set_bits(length);
    if x87::equal_or_unordered(length, course.numbers.zero.bits()) {
        for n in 0..3 {
            out.set_bits(n, 0);
        }
        return;
    }
    for n in 0..3 {
        out.set_bits(n, x87::div(from.bits(n), length));
    }
    for n in 0..3 {
        if out.bits(n) == 0x8000_0000 {
            out.set_bits(n, 0);
        }
    }
}

/// `scene::cross_product` (0x420F8D): ebx = `b`, ecx = `c`, edx = `out`, written part
/// by part.
pub fn cross_product(
    b: &(impl Cells + ?Sized),
    c: &(impl Cells + ?Sized),
    out: &(impl Cells + ?Sized),
) {
    let part =
        |x: usize, y: usize| x87::product_difference(c.bits(x), b.bits(y), b.bits(x), c.bits(y));
    out.set_bits(0, part(2, 1));
    out.set_bits(1, part(0, 2));
    out.set_bits(2, part(1, 0));
}

/// The squared rows' work (`flt_5C6FA4` ..: 16 floats over three fields, as one run).
struct Squared<'a>(&'a Course);

impl Cells for Squared<'_> {
    fn bits(&self, n: usize) -> u32 {
        let course = self.0;
        match n {
            0..4 => course.lists.squared_row_0[n].bits(),
            4..8 => course.lists.squared_row_1[n - 4].bits(),
            _ => course.lists.squared_row_2_and_after[n - 8].bits(),
        }
    }
    fn set_bits(&self, n: usize, bits: u32) {
        let course = self.0;
        match n {
            0..4 => course.lists.squared_row_0[n].set_bits(bits),
            4..8 => course.lists.squared_row_1[n - 4].set_bits(bits),
            _ => course.lists.squared_row_2_and_after[n - 8].set_bits(bits),
        }
    }
}

/// `scene::square_matrix` (0x420E94): a turning's three rows made one long and at right
/// angles again, from its third row and its second.
pub fn square_matrix(course: &Course, at: &(impl Cells + ?Sized)) {
    let squared = Squared(course);
    let (first, second, third) = (Off(&squared, 0), Off(&squared, 4), Off(&squared, 8));
    unit_vector(course, &Off(at, 8), &third);
    cross_product(&Off(at, 4), &third, &first);
    unit_vector(course, &first, &first);
    cross_product(&third, &first, &second);
    for n in 0..0x10 {
        at.set_bits(n, squared.bits(n));
    }
}

/// `scene::invert_turn` (0x420CC2): the turning at `from` undone, into `out`: its nine
/// floats' cofactors laid out the other way round, then squared.
pub fn invert_turn(course: &Course, from: &(impl Cells + ?Sized), out: &(impl Cells + ?Sized)) {
    let mut m = [0u32; 9];
    for (n, value) in m.iter_mut().enumerate() {
        *value = x87::loaded(from.bits((n / 3) * 4 + n % 3));
    }
    crate::session::write_bits(&course.lists.invert_copy, m);
    let [a, b, c, d, e, f, g, h, i] = m;
    let plain = x87::product_difference;
    let negated = x87::negated_product_difference;
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
    crate::session::write_bits(&course.lists.invert_made, made);
    for (n, value) in made.into_iter().enumerate() {
        out.set_bits((n / 3) * 4 + n % 3, x87::loaded(value));
    }
    square_matrix(course, out);
}

/// `scene::set_view_matrices` (0x438FBF).
pub fn set_view_matrices(course: &Course, frame: &crate::race::owner_frame::Frame) {
    crate::session::write_bits(
        &frame.matrices.view_turn_matrix,
        crate::session::read_bits(&frame.matrices.identity_matrix),
    );
    let angles: [u32; 3] =
        crate::session::read_first(&frame.view_and_records.view_vars.view_angles);
    set_fine_turn(course, &frame.matrices, Axis::Y, angles[0].wrapping_neg());
    set_fine_turn(course, &frame.matrices, Axis::X, angles[1].wrapping_neg());
    set_fine_turn(course, &frame.matrices, Axis::Z, angles[2].wrapping_neg());
    matrix::multiply_matrices(
        &frame.matrices.turn_y_matrix,
        &frame.matrices.turn_x_matrix,
        &frame.matrices.lit_matrix,
    );
    matrix::multiply_matrices(
        &frame.matrices.lit_matrix,
        &frame.matrices.turn_z_matrix,
        &frame.matrices.view_turn_matrix,
    );
    let place = crate::session::read_bits::<_, 3>(&frame.view_and_records.view_vars.view_place)
        .map(u32::wrapping_neg);
    matrix::set_place(&frame.matrices, place);
    crate::session::write_bits(
        &frame.matrices.view_place_matrix,
        crate::session::read_bits(&frame.matrices.placed_matrix),
    );
    matrix::multiply_matrices(
        &frame.matrices.placed_matrix,
        &frame.matrices.view_turn_matrix,
        &frame.matrices.view_matrix,
    );
    invert_turn(
        course,
        &frame.matrices.view_turn_matrix,
        &frame.matrices.view_back_matrix,
    );
}

// ---- the converted versions (`rc2k_core::scene::matrices`) ----
