//! `marks::place_shape_points` (0x4BA0AE): see `marks/splines.rs`' `place_shape_points`.
//!
//! In:  ebp a spline record (`g_marksSplinesB` / `g_marksSplinesD`)
//! Out: see `place_shape_points`
//!
//! Callers: 0x4B9968, 0x4B9979 (`sub_4B98FD`). Refuses more than 0x80 points, an angle
//! `fsincos` cannot take, and a null point pointer, before anything. Self-test: a record
//! of the test's (`TestSpline`) whose pointers name a buffer of the test's, several point
//! counts and seeds; the work variables, the record and the buffer compared.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::marks::splines::{MOST_POINTS, SPLINE_LEN, place_shape_points, work_regions};
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, state_entry};

const COUNT: u32 = 0x50;
const POINTERS: u32 = 0x454;

/// What it may write: the work variables, the record, and the points' targets.
pub fn regions(spline: u32) -> Vec<Region> {
    let mut r = work_regions();
    r.push((spline, SPLINE_LEN as usize));
    let n =
        unsafe { ((spline + COUNT) as *const i32).read_unaligned() }.clamp(0, MOST_POINTS as i32);
    for k in 0..n as u32 {
        let p = unsafe { ((spline + POINTERS + 4 * k) as *const u32).read_unaligned() };
        if p != 0 {
            r.push((p, 12));
        }
    }
    r
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { place_shape_points(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
