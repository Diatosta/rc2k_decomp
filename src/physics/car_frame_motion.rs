//! `physics::car_frame_motion` (0x43DB71): the work car's motion (`dword_71A170` ..,
//! `g_car` +0x198) turned by `g_carInverseMatrix` (`scene::rotate_point`) into the car's
//! frame, stored (rounded to floats) at `dword_71A17C`, `dword_71A180`, `g_carSpeed`
//! (+0x1A4 .. +0x1AC). The tail of `physics::apply_force` on the work car.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `sub_43BD8B` (its tail).
//! Self-test: a copy of the original (the motion, the result and the matrix the test's,
//! its call kept) against the Rust core, on values by lot under three precisions.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;

/// The Rust core: the motion at `from` turned by the matrix at `m` into `to`.
pub unsafe fn car_frame_motion_at(from: u32, m: u32, to: u32) {
    unsafe {
        let p = from as *const u32;
        let point = [
            extended(p.read_unaligned()),
            extended(p.add(1).read_unaligned()),
            extended(p.add(2).read_unaligned()),
        ];
        let out = rotate_point(m, point);
        let q = to as *mut u32;
        for (k, e) in out.iter().enumerate() {
            q.add(k).write_unaligned(rounded(*e));
        }
    }
}
