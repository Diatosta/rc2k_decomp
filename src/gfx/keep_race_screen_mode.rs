//! Keeps the given mode as the race's, and tries its pixel kind (the second byte) with 800 x 600 by
//! its size bits and the menus' other bits (`gfx::try_screen_mode`): if that works it becomes the
//! menus' mode.

use crate::errors::Refused;
use crate::framework::CarryZeroFlags;
use crate::gfx::owner::Gfx;

/// Keeps the race's mode `mode`: the flags it leaves (`try_screen_mode`'s).
pub fn keep_race_screen_mode(
    gfx: &Gfx,
    system: &crate::system::owner::System,
    d3d: &crate::d3d::owner::D3d,
    mode: u32,
) -> Result<CarryZeroFlags, Refused> {
    let display = &gfx.display;
    display.mode.race_mode.set(mode);
    let menu = rc2k_core::gfx::screen_modes::menu_mode_for(mode);
    let flags = crate::gfx::try_screen_mode::try_screen_mode(gfx, system, d3d, menu)?;
    if !flags.carry {
        display.mode.menu_mode.set(menu);
    }
    Ok(flags)
}
