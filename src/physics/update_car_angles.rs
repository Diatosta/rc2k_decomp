//! `physics::update_car_angles` (0x43BEDC): `physics::car_angles` when `dword_71A6C8` is
//! set or `dword_71A22C` is 0; else nothing.
//!
//! In:  nothing
//! Out: when run, esi = `g_crashMatrix`; the rest kept; no flags read (the callers' next
//!      instructions are `lea` and `mov`)
//!
//! Callers: `sub_43B635` (0x43B83B, 0x43B8B4, after `race::invert_car_matrix`).
//! Self-test: a copy of the original (its switches the test's, its call to
//! `car_angles`' test copy) on the four switch settings.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::physics::car_angles::car_angles;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const FORCE_RVA: usize = rva(0x71A6C8); // dword_71A6C8
const HOLD_RVA: usize = rva(0x71A22C); // dword_71A22C

use rc2k_core::physics::motion::angles_wanted as wanted;

/// The Rust version.
// Converted: `rc2k_core::physics::motion::angles_wanted`.
pub unsafe fn update_car_angles() -> Left {
    let (force, hold) = unsafe { (game_read::<u32>(FORCE_RVA), game_read::<u32>(HOLD_RVA)) };
    if wanted(force, hold) {
        unsafe { car_angles() }
    } else {
        Left::nothing()
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(unsafe { update_car_angles() })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
