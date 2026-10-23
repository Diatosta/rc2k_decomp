//! `menu::reset_drivers` (0x470399): every driver (`g_drivers`, 0x94 each) given +0x24 1,
//! +0xC 0 and +0x10 0 (a reading: in the event, no points, no place).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers and `sub_4A9C9B`. Refuses a table without its end entry,
//! before anything. Self-test: the original and the Rust version on the game's drivers;
//! put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn reset_drivers(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::event::DRIVER_RESET`, stored in each driver.
    driver_table()?.store_all(&rc2k_core::menu::event::DRIVER_RESET);
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { reset_drivers(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
