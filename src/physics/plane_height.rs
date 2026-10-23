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
//! Self-test: a copy of the original on test variables, against the converted version,
//! values by lot (some large), exact at double precision.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;
use rc2k_core::physics::ground::plane_height as plane_height_core;

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
        let p = |x: usize| game_addr(rva(x)) as u32;
        Self {
            a: p(0x639718),
            b: p(0x639724),
            c: p(0x63970C),
            f: p(0x639810),
            g: p(0x639814),
            out: p(0x639828),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
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

/// The converted version (`rc2k_core::physics::ground`, plain floats) on the variables
/// at `p`: the hook's.
// Converted: `rc2k_core::physics::ground::plane_height`.
pub unsafe fn plane_height_plain_at(p: &Plane) {
    unsafe {
        let h = plane_height_core(
            at(p.a) as i32,
            at(p.b) as i32,
            at(p.c) as i32,
            f32::from_bits(at(p.f)),
            f32::from_bits(at(p.g)),
        );
        (p.out as *mut i32).write_unaligned(h);
    }
}

/// The Rust version (the exact one, for `physics::ground`, `face_under_point` and the
/// race's shadows).
pub unsafe fn plane_height() {
    unsafe { plane_height_at(&Plane::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { plane_height_plain_at(&Plane::game()) };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
