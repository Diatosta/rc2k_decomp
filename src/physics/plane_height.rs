//! `physics::plane_height` (0x4425B6): `dword_639828` = c + (a - c) `flt_639810` + (b -
//! c) `flt_639814`, `fistp`'d (the control word's rounding), with a, b, c the whole
//! numbers `dword_639718`, `dword_639724`, `dword_63970C`: by the look of it the height
//! of a course face's plane under a point, from its corners' heights and the point's
//! shares (a reading; set by `sub_442006`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Callers (2): `sub_442006` (the face under a point, the ground's search) and
//! `sub_457B86`.

use crate::framework::{game_addr, rva};
use crate::physics::ground_store::GROUND_WORK;

/// The variables' addresses: the game's, or a test's.
#[derive(Clone, Copy)]
pub struct Plane {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub f: u32,
    pub g: u32,
    pub out: u32,
}

impl Plane {
    pub fn game() -> Self {
        let _p = |x: usize| game_addr(rva(x)) as u32;
        Self {
            a: (GROUND_WORK.face_corners[4].addr() as u32),
            b: (GROUND_WORK.face_corners[7].addr() as u32),
            c: (GROUND_WORK.face_corners[1].addr() as u32),
            f: (GROUND_WORK.plane_f.addr() as u32),
            g: (GROUND_WORK.plane_g.addr() as u32),
            out: (GROUND_WORK.ground_height.addr() as u32),
        }
    }
}

fn at(a: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(a, "physics::plane_height's operand")
}

/// The Rust core on the variables at `p`.
pub unsafe fn plane_height_at(p: &Plane) {
    unsafe {
        let h = crate::math::x87_run::plane_height(
            at(p.a) as i32,
            at(p.b) as i32,
            at(p.c) as i32,
            at(p.f),
            at(p.g),
        );
        (p.out as *mut i32).write_unaligned(h);
    }
}

/// The Rust version (the exact one, for `physics::ground`, `face_under_point` and the
/// race's shadows).
pub unsafe fn plane_height() {
    unsafe { plane_height_at(&Plane::game()) }
}
