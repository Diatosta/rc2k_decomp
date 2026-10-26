//! The view's matrices and the functions that make them: shared by their eight functions.
//!
//! The three fine turnings take an angle of 32 bits a turn and work the sine and
//! cosine out (`fsincos`), where `scene::set_turn_x` and `_y` look sixteen bits up in
//! tables; the first two write the same matrices as those.

use crate::math::x87;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::matrices_store::MATRICES;
use crate::race::numbers_store::NUMBERS;
use crate::race::view_vars_store::VIEW_VARS;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix;
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
pub fn set_fine_turn(axis: Axis, angle: u32) {
    DRAW.scratch.set_bits(angle);
    let (sine, cosine) = x87::sine_cosine(
        angle as i32,
        NUMBERS.k65536.bits(),
        NUMBERS.one_over_two_pi.bits(),
    );
    let negated = sine ^ 0x8000_0000;
    // The work pair, then where the cosine, the sine, the sine negated and the
    // cosine go in the matrix.
    let (work, rows, places): (&[Float; 2], &[Float; 16], [usize; 4]) = match axis {
        Axis::X => (&DRAW.turn_x_work, &MATRICES.turn_x_matrix, [5, 6, 9, 10]),
        Axis::Y => (&DRAW.turn_y_work, &MATRICES.turn_y_matrix, [0, 8, 2, 10]),
        Axis::Z => (&DRAW.turn_z_work, &MATRICES.turn_z_matrix, [0, 1, 4, 5]),
    };
    crate::session::write_bits(work, [sine, cosine]);
    for (place, value) in places.into_iter().zip([cosine, sine, negated, cosine]) {
        rows[place].set_bits(value);
    }
}

fn float(at: usize) -> u32 {
    crate::pointers::read_or_end::<u32>((at) as u32, "scene::view_matrix's operand")
}

fn put(at: usize, value: u32) {
    crate::pointers::write_or_end::<u32>((at) as u32, value, "scene::view_matrix's operand")
}

/// `scene::unit_vector` (0x420EF8): three floats made one long, into `out`; all 0 when
/// their length compares equal to `flt_5C5D90` (0, or no number). Minus zero made zero.
/// The length is left in `dword_62400C`.
pub fn unit_vector(from: usize, out: usize) {
    let length = x87::length(crate::pointers::host_at(from) as *const u32);
    DRAW.scratch.set_bits(length);
    if x87::equal_or_unordered(length, NUMBERS.zero.bits()) {
        for n in 0..3 {
            put(out + n * 4, 0);
        }
        return;
    }
    for n in 0..3 {
        put(out + n * 4, x87::div(float(from + n * 4), length));
    }
    for n in 0..3 {
        if float(out + n * 4) == 0x8000_0000 {
            put(out + n * 4, 0);
        }
    }
}

/// `scene::cross_product` (0x420F8D): ebx = `b`, ecx = `c`, edx = `out`, written part
/// by part.
pub fn cross_product(b: usize, c: usize, out: usize) {
    let part = |x: usize, y: usize| {
        x87::product_difference(
            float(c + x * 4),
            float(b + y * 4),
            float(b + x * 4),
            float(c + y * 4),
        )
    };
    put(out, part(2, 1));
    put(out + 4, part(0, 2));
    put(out + 8, part(1, 0));
}

/// `scene::square_matrix` (0x420E94): a turning's three rows made one long and at right
/// angles again, from its third row and its second.
pub fn square_matrix(at: usize) {
    let (first, second, third) = (
        COURSE_LISTS.squared_row_0[0].addr(),
        COURSE_LISTS.squared_row_0[0].addr() + 0x10,
        COURSE_LISTS.squared_row_0[0].addr() + 0x20,
    );
    unit_vector(at + 0x20, third);
    cross_product(at + 0x10, third, first);
    unit_vector(first, first);
    cross_product(third, first, second);
    for n in 0..0x10 {
        put(at + n * 4, float(first + n * 4));
    }
}

/// `scene::invert_turn` (0x420CC2): the turning at `from` undone, into `out`: its nine
/// floats' cofactors laid out the other way round, then squared.
pub fn invert_turn(from: usize, out: usize) {
    let mut m = [0u32; 9];
    for (n, value) in m.iter_mut().enumerate() {
        *value = x87::loaded(float(from + (n / 3) * 0x10 + (n % 3) * 4));
    }
    crate::session::write_bits(&COURSE_LISTS.invert_copy, m);
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
    crate::session::write_bits(&COURSE_LISTS.invert_made, made);
    for (n, value) in made.into_iter().enumerate() {
        put(out + (n / 3) * 0x10 + (n % 3) * 4, x87::loaded(value));
    }
    square_matrix(out);
}

/// `scene::set_view_matrices` (0x438FBF).
pub fn set_view_matrices() {
    crate::session::write_bits(
        &MATRICES.view_turn_matrix,
        crate::session::read_bits(&MATRICES.identity_matrix),
    );
    let angles: [u32; 3] = crate::session::read_first(&VIEW_VARS.view_angles);
    set_fine_turn(Axis::Y, angles[0].wrapping_neg());
    set_fine_turn(Axis::X, angles[1].wrapping_neg());
    set_fine_turn(Axis::Z, angles[2].wrapping_neg());
    matrix::multiply_matrices(
        MATRICES.turn_y_matrix[0].addr(),
        MATRICES.turn_x_matrix[0].addr(),
        MATRICES.lit_matrix[0].addr(),
    );
    matrix::multiply_matrices(
        MATRICES.lit_matrix[0].addr(),
        MATRICES.turn_z_matrix[0].addr(),
        MATRICES.view_turn_matrix[0].addr(),
    );
    let place = crate::session::read_bits::<_, 3>(&VIEW_VARS.view_place).map(u32::wrapping_neg);
    matrix::set_place(place);
    crate::session::write_bits(
        &MATRICES.view_place_matrix,
        crate::session::read_bits(&MATRICES.placed_matrix),
    );
    matrix::multiply_matrices(
        MATRICES.placed_matrix[0].addr(),
        MATRICES.view_turn_matrix[0].addr(),
        MATRICES.view_matrix[0].addr(),
    );
    invert_turn(
        MATRICES.view_turn_matrix[0].addr(),
        MATRICES.view_back_matrix[0].addr(),
    );
}

// ---- the converted versions (`rc2k_core::scene::matrices`) ----
