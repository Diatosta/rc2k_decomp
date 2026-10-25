//! `physics::matrix_angles` (0x4212C3): three angles of the turn matrix at esi. The
//! matrix turns (0, 0, 1) (`scene::rotate_point`, stored as floats in `flt_5C70E0`,
//! `E4`, `E8`): `flt_5C70D8` = `fpatan` of E0 over E8; the same turn again, and
//! `flt_5C70D4` = minus `physics::clamped_asin` of E4; then (1, 0, 0) turned, and
//! `flt_5C70DC` = `clamped_asin` of E4 over the cosine of D4 (80 bits). A reading: the
//! car's heading (D8), pitch (D4) and roll (DC).
//!
//! In:  esi the matrix
//! Out: every register kept; the x87 stack as it was; no flags read
//!
//! Callers: `physics::car_angles` (`g_crashMatrix`).

use crate::framework::{game_addr, rva};
use crate::math::x87_run::{neg_rounded, over_cos_80, patan};
use crate::physics::clamped_asin::clamped_asin_at;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_store::DRAW;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;

const TURNED_RVA: usize = rva(0x5C70E0); // flt_5C70E0, E4, E8
const ANGLES_RVA: usize = rva(0x5C70D4); // flt_5C70D4 (pitch), D8 (heading), DC (roll)

/// Where the function's variables are: the game's or a test's.
#[derive(Clone, Copy)]
pub struct AnglePlaces {
    /// The turned unit vector (three floats).
    pub turned: u32,
    /// The three angles.
    pub angles: u32,
    pub scratch: u32,
    pub limit: u32,
}

impl AnglePlaces {
    pub fn game() -> Self {
        Self {
            turned: game_addr(TURNED_RVA) as u32,
            angles: game_addr(ANGLES_RVA) as u32,
            scratch: DRAW.scratch.addr() as u32,
            limit: NUMBERS.one.addr() as u32,
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// `rotate_point` of the unit vector `unit` by `m`, stored as floats at `p.turned`.
unsafe fn turn_unit(m: u32, p: &AnglePlaces, unit: [u32; 3]) {
    unsafe {
        let out = rotate_point(m, unit.map(extended));
        for (k, e) in out.iter().enumerate() {
            put(p.turned + 4 * k as u32, rounded(*e));
        }
    }
}

/// The Rust core: the matrix at `m`, the variables at `p`.
pub unsafe fn matrix_angles_at(m: u32, p: &AnglePlaces) {
    const ONE: u32 = 0x3F80_0000;
    unsafe {
        turn_unit(m, p, [0, 0, ONE]);
        put(p.angles + 4, patan(at(p.turned), at(p.turned + 8)));
        turn_unit(m, p, [0, 0, ONE]);
        let pitch = clamped_asin_at(extended(at(p.turned + 4)), p.scratch, p.limit);
        put(p.angles, neg_rounded(pitch));
        turn_unit(m, p, [ONE, 0, 0]);
        let sine = over_cos_80(at(p.turned + 4), at(p.angles));
        let roll = clamped_asin_at(sine, p.scratch, p.limit);
        put(p.angles + 8, rounded(roll));
    }
}
