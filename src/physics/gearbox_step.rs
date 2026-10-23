//! `physics::gearbox_step` (0x43C15C): the gearbox's turn. Unless `g_car` +0xA8 is set,
//! or (with `dword_62389C` not 1) the start timer +0x14 is below 0x320 (by its sign): in
//! grade mode 2 `physics::manual_gearbox`; with a gear asked for (+0x468 bits 8 .. 11
//! not 0xE) `physics::take_asked_gear` and it ends there; else by +0x38 (`unk_71A010`)
//! 1 `manual_gearbox`, 2 `physics::auto_gearbox`, other `manual_gearbox` (by
//! `physics::manual_gearbox_call`). Then, after a change (+0x238 1), the engine speed x
//! `flt_613E9C`; `physics::wheels_from_revs`.
//!
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read (the callers return)
//!
//! Callers: `sub_43CB14` (under `sub_43B635`), `sub_421848`.
//! Refuses what the steps refuse; what was written is put back.
//! Self-test: `physics/gearbox.rs` (`test_gearbox`), with gears asked for and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::math::x87;
use crate::physics::auto_gearbox::auto_gearbox_in;
use crate::physics::gearbox::*;
use crate::physics::manual_gearbox::manual_gearbox_in;
use crate::physics::take_asked_gear::take_asked_gear;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The steps on `r`.
// Converted: `rc2k_core::physics::gearbox::gearbox_turn`.
pub fn gearbox_step_in(r: &mut Regs) -> Result<(), Refused> {
    use rc2k_core::physics::gearbox::{GearboxTurn, gearbox_turn};
    let turn = gearbox_turn(
        g(0x71A080),
        || g(0x62389C),
        || g(0x719FEC),
        || g(0x71A234),
        || g(0x71A440),
        || g(0x71A010),
    );
    match turn {
        GearboxTurn::Idle => {}
        GearboxTurn::Manual => manual_gearbox_in(r)?,
        GearboxTurn::Asked => {
            unsafe { take_asked_gear() }?.apply(r);
            return Ok(());
        }
        GearboxTurn::Auto => auto_gearbox_in(r)?,
    }
    if g(CHANGED) == 1 {
        p(ENGINE, x87::mul(g(ENGINE), g(0x613E9C)));
    }
    wheels(r)
}

/// The Rust version.
pub unsafe fn gearbox_step(entry: &Regs) -> Result<Left, Refused> {
    guarded(entry, &gearbox_step_in)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { gearbox_step(r) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
