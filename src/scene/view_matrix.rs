//! The view's matrices and the functions that make them: shared by their eight hooks.
//!
//! The three fine turnings take an angle of 32 bits a turn and work the sine and
//! cosine out (`fsincos`), where `scene::set_turn_x` and `_y` look sixteen bits up in
//! tables; the first two write the same matrices as those.

use crate::framework::{EAX, EBP, EBX, ECX, EDI, EDX, ESI, game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::matrix::{
    self, LIT_RVA, PLACED_RVA, TURN_X_RVA, TURN_X_WORK_RVA, TURN_Y_RVA, TURN_Y_WORK_RVA,
    VIEW_TURN_RVA,
};
use crate::screens::harness::Left;

pub const TURN_Z_RVA: usize = rva(0x5EA00C); // flt_5EA00C
pub const TURN_Z_WORK_RVA: usize = rva(0x624158); // dword_624158, dword_62415C
pub const IDENTITY_RVA: usize = rva(0x5E9F4C); // unk_5E9F4C
pub const KEPT_PLACE_RVA: usize = rva(0x5EA1CC); // unk_5EA1CC
pub const VIEW_BACK_RVA: usize = rva(0x5EA20C); // flt_5EA20C, the view's turning undone
pub const VIEW_RVA: usize = rva(0x5EA2CC); // g_viewMatrix
pub const VIEW_PLACE_RVA: usize = rva(0x606298); // dword_606298, dword_60629C, dword_6062A0
pub const VIEW_ANGLES_RVA: usize = rva(0x6062B8); // dword_6062B8 (y), dword_6062BC (x), dword_6062C0 (z)

pub const HALF_TURNS_RVA: usize = rva(0x5C5D98); // flt_5C5D98, 65536.0
pub const TURN_RVA: usize = rva(0x5C5DD4); // flt_5C5DD4, 1 / (2 pi)
pub const NOTHING_RVA: usize = rva(0x5C5D90); // flt_5C5D90, 0.0
/// The squaring's work: three rows of four floats, `unk_5C6FD0`, `unk_5C6FE0`,
/// `unk_5C6FF0`; the copy back takes the four dwords after them along.
pub const SQUARED_RVA: usize = rva(0x5C6FD0);
/// The inverting's work: the nine floats copied (`flt_5C7050` ..) and the nine made
/// (`flt_5C7074` ..).
pub const INVERTED_RVA: usize = rva(0x5C7050);
/// Which of the three turnings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub fn matrix(self) -> usize {
        match self {
            Axis::X => TURN_X_RVA,
            Axis::Y => TURN_Y_RVA,
            Axis::Z => TURN_Z_RVA,
        }
    }

    pub fn work(self) -> usize {
        match self {
            Axis::X => TURN_X_WORK_RVA,
            Axis::Y => TURN_Y_WORK_RVA,
            Axis::Z => TURN_Z_WORK_RVA,
        }
    }

    /// Where the cosine, the sine, the sine negated and the cosine go.
    fn places(self) -> [usize; 4] {
        match self {
            Axis::X => [0x14, 0x18, 0x24, 0x28],
            Axis::Y => [0, 0x20, 8, 0x28],
            Axis::Z => [0, 4, 0x10, 0x14],
        }
    }
}

/// `scene::set_fine_turn_x` (0x4270BA), `scene::set_fine_turn_y` (0x42710C),
/// `scene::set_turn_z` (0x42715D).
pub unsafe fn set_fine_turn(axis: Axis, angle: u32) -> Left {
    unsafe {
        game_write(CHECKED_RVA, angle);
        let (sine, cosine) =
            x87::sine_cosine(angle as i32, game_read(HALF_TURNS_RVA), game_read(TURN_RVA));
        game_write(axis.work(), [sine, cosine]);
        let negated = sine ^ 0x8000_0000;
        for (place, value) in axis
            .places()
            .into_iter()
            .zip([cosine, sine, negated, cosine])
        {
            game_write(axis.matrix() + place, value);
        }
        Left::nothing()
            .with(EAX, sine)
            .with(EBX, cosine)
            .with(ECX, negated)
            .with(EDI, game_addr(axis.matrix()) as u32)
    }
}

unsafe fn float(at: usize) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

unsafe fn put(at: usize, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) }
}

/// `scene::unit_vector` (0x420EF8): three floats made one long, into `out`; all 0 when
/// their length compares equal to `flt_5C5D90` (0, or no number). Minus zero made zero.
/// The length is left in `dword_62400C`.
pub unsafe fn unit_vector(from: usize, out: usize) {
    unsafe {
        let length = x87::length(from as *const u32);
        game_write(CHECKED_RVA, length);
        if x87::equal_or_unordered(length, game_read(NOTHING_RVA)) {
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
}

/// `scene::cross_product` (0x420F8D): ebx = `b`, ecx = `c`, edx = `out`, written part
/// by part.
pub unsafe fn cross_product(b: usize, c: usize, out: usize) {
    unsafe {
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
}

/// `scene::square_matrix` (0x420E94): a turning's three rows made one long and at right
/// angles again, from its third row and its second.
pub unsafe fn square_matrix(at: usize) {
    unsafe {
        let (first, second, third) = (
            game_addr(SQUARED_RVA),
            game_addr(SQUARED_RVA) + 0x10,
            game_addr(SQUARED_RVA) + 0x20,
        );
        unit_vector(at + 0x20, third);
        cross_product(at + 0x10, third, first);
        unit_vector(first, first);
        cross_product(third, first, second);
        for n in 0..0x10 {
            put(at + n * 4, float(first + n * 4));
        }
    }
}

/// `scene::invert_turn` (0x420CC2): the turning at `from` undone, into `out`: its nine
/// floats' cofactors laid out the other way round, then squared.
pub unsafe fn invert_turn(from: usize, out: usize) {
    unsafe {
        let mut m = [0u32; 9];
        for (n, value) in m.iter_mut().enumerate() {
            *value = x87::loaded(float(from + (n / 3) * 0x10 + (n % 3) * 4));
        }
        game_write(INVERTED_RVA, m);
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
        game_write(INVERTED_RVA + 0x24, made);
        for (n, value) in made.into_iter().enumerate() {
            put(out + (n / 3) * 0x10 + (n % 3) * 4, x87::loaded(value));
        }
        square_matrix(out);
    }
}

/// `scene::set_view_matrices` (0x438FBF).
pub unsafe fn set_view_matrices() -> Left {
    unsafe {
        game_write(VIEW_TURN_RVA, game_read::<[u32; 16]>(IDENTITY_RVA));
        let angles: [u32; 3] = game_read(VIEW_ANGLES_RVA);
        set_fine_turn(Axis::Y, angles[0].wrapping_neg());
        set_fine_turn(Axis::X, angles[1].wrapping_neg());
        set_fine_turn(Axis::Z, angles[2].wrapping_neg());
        matrix::multiply_matrices(
            game_addr(TURN_Y_RVA),
            game_addr(TURN_X_RVA),
            game_addr(LIT_RVA),
        );
        matrix::multiply_matrices(
            game_addr(LIT_RVA),
            game_addr(TURN_Z_RVA),
            game_addr(VIEW_TURN_RVA),
        );
        let place = game_read::<[u32; 3]>(VIEW_PLACE_RVA).map(u32::wrapping_neg);
        matrix::set_place(place);
        game_write(KEPT_PLACE_RVA, game_read::<[u32; 16]>(PLACED_RVA));
        matrix::multiply_matrices(
            game_addr(PLACED_RVA),
            game_addr(VIEW_TURN_RVA),
            game_addr(VIEW_RVA),
        );
        invert_turn(game_addr(VIEW_TURN_RVA), game_addr(VIEW_BACK_RVA));
        Left::nothing()
            .with(EAX, place[0])
            .with(EBX, place[1])
            .with(ECX, 0)
            .with(EDX, 0)
            .with(ESI, game_addr(VIEW_TURN_RVA) as u32)
            .with(EDI, game_addr(VIEW_BACK_RVA) as u32)
            .with(EBP, game_addr(VIEW_RVA) as u32)
    }
}

// ---- the converted versions (`rc2k_core::scene::matrices`) ----
