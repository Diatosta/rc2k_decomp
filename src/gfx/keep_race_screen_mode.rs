//! `gfx::keep_race_screen_mode` (0x404805): mode eax kept as the race's
//! (`g_raceScreenMode`), and its pixel kind (the second byte) with 0x320002 (800 x 600 by
//! its size bits, the menus' other bits) tried (`gfx::try_screen_mode`): if that works it
//! becomes the menus' mode (`g_menuScreenMode`).
//!
//! In:  eax = the mode
//! Out: every register kept (`pusha` .. `popa`); the flags as `try_screen_mode` leaves them
//!
//! One caller, `gfx::screen_mode_choices`, which calls it directly, as it calls
//! `try_screen_mode` (2026-10-23).

use crate::errors::Refused;
use crate::framework::{Flags, Regs};
use crate::race::viewer::put;

pub const RACE_MODE: usize = 0x4F4948; // g_raceScreenMode
pub const MENU_MODE: usize = 0x4F4944; // g_menuScreenMode

/// The Rust version on the race's mode `mode`: the flags it leaves (`try_screen_mode`'s).
pub fn keep_race_screen_mode(mode: u32) -> Result<Flags, Refused> {
    put(RACE_MODE, mode);
    // Converted: `rc2k_core::gfx::screen_modes::menu_mode_for`.
    let menu = rc2k_core::gfx::screen_modes::menu_mode_for(mode);
    let flags = crate::gfx::try_screen_mode::try_screen_mode(menu)?;
    if !flags.carry {
        put(MENU_MODE, menu);
    }
    Ok(flags)
}

/// The Rust version where the typed tables run it (a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), Refused> {
    keep_race_screen_mode(r.eax)?.apply(r);
    Ok(())
}
