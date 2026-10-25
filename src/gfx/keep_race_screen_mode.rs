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
use crate::framework::Flags;
use crate::gfx::screen_state::SCREEN_MODE;

/// The Rust version on the race's mode `mode`: the flags it leaves (`try_screen_mode`'s).
pub fn keep_race_screen_mode(mode: u32) -> Result<Flags, Refused> {
    SCREEN_MODE.race_mode.set(mode);
    // Converted: `rc2k_core::gfx::screen_modes::menu_mode_for`.
    let menu = rc2k_core::gfx::screen_modes::menu_mode_for(mode);
    let flags = crate::gfx::try_screen_mode::try_screen_mode(menu)?;
    if !flags.carry {
        SCREEN_MODE.menu_mode.set(menu);
    }
    Ok(flags)
}
