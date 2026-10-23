//! `menu::reset_cars_750724` (0x46FC52): four entries of `off_750724`'s table (0x5C apart
//! from +0x118) given +0x14 0, +0x48 -0x3E4 and +0x1C -0x57 (a reading: the cars on the
//! front end's carousel at their start).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! They are the in-game options menu's items 3 .. 6 (`menu/in_game_options_items.rs`):
//! this hides all four, as `menu::hide_in_game_options` hides the last ones.
//! Callers: `sub_487E2C`, the event drivers. Self-test: the original and the Rust version
//! on the game's table; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::in_game_options_items::set;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;
use rc2k_core::menu::items::{HIDDEN, IN_GAME_OPTIONS};

// off_750724
/// The Rust version (see the header): the registers it leaves.
pub unsafe fn reset_cars_750724(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::menu::items::HIDDEN` (the four items `hide_in_game_options`
    // hides, all of them).
    for n in IN_GAME_OPTIONS {
        unsafe { set(n, HIDDEN) };
    }
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { reset_cars_750724(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
