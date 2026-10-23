//! `marks::view_step` (0x4C1593): the view's step since the last frame, negated: for each
//! of x, y, z `flt_7692EA` .. = -(`flt_76927A` .. less `flt_769296` ..) (`fld; fsub;
//! fchs; fstp`), then `flt_769296` .. = `flt_76927A` .. (`push`/`pop`).
//!
//! In:  nothing
//! Out: every register kept
//!
//! Caller: `sub_4C1270`. Self-test: the six inputs by lot, the original and the Rust
//! version compared on the six results; put back afterwards.
//!
//! Converted 2026-10-21: the hook's Rust version is `view_step_plain`, through
//! `rc2k_core::marks::view::step` (plain floats; the x87 version stays for the Rust callers
//! in unconverted code). Shadow mode and the self-test compare within `service::floats`'
//! tolerances (the self-test at both test precisions, exact at double).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::math::x87;
use crate::screens::harness::Left;
use crate::service::floats::*;
use crate::service::harness::{Region, at};

const NOW_RVA: usize = rva(0x76927A); // flt_76927A .. flt_769282
const LAST_RVA: usize = rva(0x769296); // flt_769296 .. flt_76929E
const STEP_RVA: usize = rva(0x7692EA); // flt_7692EA .. flt_7692F2

/// The Rust version (see the header).
pub unsafe fn view_step() {
    unsafe {
        for k in 0..3 {
            let (now, last) = (
                game_read::<u32>(NOW_RVA + 4 * k),
                game_read::<u32>(LAST_RVA + 4 * k),
            );
            game_write(STEP_RVA + 4 * k, x87::negated_sub(now, last));
        }
        for k in 0..3 {
            game_write(LAST_RVA + 4 * k, game_read::<u32>(NOW_RVA + 4 * k));
        }
    }
}

/// The converted version (plain floats): the hook's.
// Converted: `rc2k_core::marks::view::step`
pub unsafe fn view_step_plain() {
    unsafe {
        let now: [u32; 3] = game_read(NOW_RVA);
        let last: [u32; 3] = game_read(LAST_RVA);
        let step = rc2k_core::marks::view::step(now.map(f32::from_bits), last.map(f32::from_bits));
        game_write(STEP_RVA, step.map(f32::to_bits));
        game_write(LAST_RVA, now);
    }
}

pub fn regions() -> Vec<Region> {
    (0..3)
        .flat_map(|k| [at(STEP_RVA + 4 * k), at(LAST_RVA + 4 * k)])
        .collect()
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { view_step_plain() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
