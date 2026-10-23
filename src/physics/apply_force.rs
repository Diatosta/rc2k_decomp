//! `physics::apply_force` (0x45FAE1): one force record (esi, its first three floats) on a
//! car (edi, its block): copied to `flt_614B30` .., times the step `flt_61388C`
//! (`physics::scale_vector`; the step, left in st(0), then rounded into `dword_62400C`),
//! turned into the world by `g_crashMatrix` (`physics::turn_by_car_matrix`), added to the
//! car's motion (+0x198 .. +0x1A0); then the motion turned by the car's inverse matrix
//! (+0x9CC, `scene::rotate_point`) into +0x1A4 .. +0x1AC (the last `g_carSpeed`). All in
//! `pusha` / `popa`. A reading: a force's push on the car.
//!
//! In:  esi the record, edi the car block
//! Out: every register kept; no flags read
//!
//! Caller: `physics::apply_forces` (in a loop over records).
//! Self-test: a copy of the original (its scratch the test's, its three calls kept) on a
//! test record and car block, against the converted version: exact at double precision,
//! within `SUM` at extended.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, rva};
use crate::math::x87;
use crate::physics::scale_vector::scale_vector;
use crate::physics::turn_by_car_matrix::turn_by_car_matrix;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::scene::vector3::{read_floats, write_floats};
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

unsafe fn put(a: u32, v: u32) {
    unsafe { (a as *mut u32).write_unaligned(v) }
}

/// The Rust core: the record at `rec`, the car at `car`, the work vector at `work`, the
/// scratch at `scratch`.
pub unsafe fn apply_at(rec: u32, car: u32, work: u32, scratch: u32) {
    unsafe {
        for k in 0..3 {
            put(work + 4 * k, at(rec + 4 * k));
        }
        let dt = extended(game_read(rva(0x61388C)));
        scale_vector(work, dt);
        put(scratch, rounded(dt));
        turn_by_car_matrix(work);
        for k in 0..3 {
            let o = car + 0x198 + 4 * k;
            put(o, x87::add(at(work + 4 * k), at(o)));
        }
        let p = [
            extended(at(car + 0x198)),
            extended(at(car + 0x19C)),
            extended(at(car + 0x1A0)),
        ];
        let out = rotate_point(car + 0x9CC, p);
        for (k, e) in out.iter().enumerate() {
            put(car + 0x1A4 + 4 * k as u32, rounded(*e));
        }
    }
}

/// The converted version (`rc2k_core::physics::pushes`, plain floats): the record at
/// `rec`, the car at `car`, the work vector at `work`, the scratch at `scratch`.
// Converted: `rc2k_core::physics::pushes::apply_force`.
pub unsafe fn apply_at_plain(rec: u32, car: u32, work: u32, scratch: u32) {
    unsafe {
        let crash = game_addr(rva(0x71A08C)) as u32; // g_crashMatrix
        let push = rc2k_core::physics::pushes::apply_force(
            read_floats(rec),
            f32::from_bits(game_read(rva(0x61388C))),
            &read_floats(crash),
            read_floats(car + 0x198),
            &read_floats(car + 0x9CC),
        );
        write_floats(work, &push.work);
        write_floats(scratch, &[push.step]);
        write_floats(car + 0x198, &push.motion);
        write_floats(car + 0x1A4, &push.local);
    }
}

/// The Rust version (the exact one, for `physics::apply_forces`).
pub unsafe fn apply_force(rec: u32, car: u32) {
    unsafe {
        apply_at(
            rec,
            car,
            game_addr(rva(0x614B30)) as u32,
            game_addr(rva(0x62400C)) as u32,
        )
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe {
        apply_at_plain(
            r.esi,
            r.edi,
            game_addr(rva(0x614B30)) as u32,
            game_addr(rva(0x62400C)) as u32,
        )
    };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
