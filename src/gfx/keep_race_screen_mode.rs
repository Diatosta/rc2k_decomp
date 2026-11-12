//! `gfx::keep_race_screen_mode` (0x404805): the given mode kept as the race's
//! (`g_raceScreenMode`), and its pixel kind (the second byte) with 0x320002 (800 x 600 by
//! its size bits, the menus' other bits) tried (`gfx::try_screen_mode`): if that works it
//! becomes the menus' mode (`g_menuScreenMode`).
//!
//! Takes: the mode
//!
//! One caller, `gfx::screen_mode_choices`, which calls it directly, as it calls
//! `try_screen_mode`.

use crate::errors::Refused;
use crate::framework::CarryZeroFlags;
use crate::gfx::owner::Gfx;

/// Keeps the race's mode `mode`: the flags it leaves (`try_screen_mode`'s).
pub fn keep_race_screen_mode(gfx: &Gfx, mode: u32) -> Result<CarryZeroFlags, Refused> {
    let display = &gfx.display;
    display.mode.race_mode.set(mode);
    // Converted: `rc2k_core::gfx::screen_modes::menu_mode_for`.
    let menu = rc2k_core::gfx::screen_modes::menu_mode_for(mode);
    let flags = crate::gfx::try_screen_mode::try_screen_mode(gfx, menu)?;
    if !flags.carry {
        display.mode.menu_mode.set(menu);
    }
    Ok(flags)
}
