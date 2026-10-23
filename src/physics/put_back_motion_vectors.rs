//! `physics::put_back_motion_vectors` (0x43BD8B): the three dwords at `dword_614BCC` ..
//! back into the work car's motion (`dword_71A170` .., `g_car` +0x198) and those at
//! `dword_614BDC` .. into its spin (`dword_71A15C` .., +0x184), eax pushed around; then
//! `physics::car_frame_motion`. The undoing of `physics::keep_motion_vectors`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43B635` (0x43B8F1, before `sub_43F7AC`).
//! Self-test: a copy of the original (all its variables the test's, its call to
//! `car_frame_motion`'s test copy on the same) against the Rust core, values by lot.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::physics::car_frame_motion::car_frame_motion_at;
use crate::physics::turn_by_inverse::INVERSE_RVA;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// Where the vectors are: the game's or a test's.
#[derive(Clone, Copy)]
pub struct MotionPlaces {
    pub kept_motion: u32,
    pub kept_spin: u32,
    pub motion: u32,
    pub spin: u32,
    pub local: u32,
    pub inverse: u32,
}

impl MotionPlaces {
    pub fn game() -> Self {
        let g = |a: usize| game_addr(rva(a)) as u32;
        Self {
            kept_motion: g(0x614BCC),
            kept_spin: g(0x614BDC),
            motion: g(0x71A170),
            spin: g(0x71A15C),
            local: g(0x71A17C),
            inverse: game_addr(INVERSE_RVA) as u32,
        }
    }
}

/// The Rust core.
pub unsafe fn put_back_motion_vectors_at(p: &MotionPlaces) {
    unsafe {
        for k in 0..3usize {
            let m = (p.kept_motion as *const u32).add(k).read_unaligned();
            (p.motion as *mut u32).add(k).write_unaligned(m);
        }
        for k in 0..3usize {
            let s = (p.kept_spin as *const u32).add(k).read_unaligned();
            (p.spin as *mut u32).add(k).write_unaligned(s);
        }
        car_frame_motion_at(p.motion, p.inverse, p.local);
    }
}

/// The Rust version.
pub unsafe fn put_back_motion_vectors() {
    unsafe { put_back_motion_vectors_at(&MotionPlaces::game()) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { put_back_motion_vectors() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
