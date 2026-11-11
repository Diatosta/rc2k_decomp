//! `gfx::try_screen_mode` (0x404AED): whether the given screen mode works: the mode's entry
//! looked up in the mode table (`g_screenModeTable`, `g_screenModeCount`, by its bits but
//! the third byte's low nibble's: `& 0xF0FFFF`, 0 failing), the mode set
//! (`gfx::set_screen_mode`), and with `g_screenModeFlags` bit 0x100000 a trial of the
//! textures (`gfx::allocate_textures` for 0x40 x 0x40 textures, then
//! `gfx::release_all_textures`). Meanwhile `trying` is 1 and the reports
//! (`reports.on`) off, both put back after.
//!
//! Takes: the mode
//!
//! Callers: `gfx::choose_screen_modes` (its list of modes), `gfx::screen_mode_choices`,
//! `gfx::keep_race_screen_mode`, and two in the race's set-up (0x442D2C, 0x4432B6). Its
//! three calls (set the mode, allocate, release: the platform's) are direct.

use crate::errors::Refused;
use crate::framework::CarryZeroFlags;
use crate::gfx::owner::Display;
use crate::gfx::owner::gfx;

/// Tries `mode`: the flags it leaves (carry: it does not work).
pub fn try_screen_mode(display: &Display, mode: u32) -> Result<CarryZeroFlags, Refused> {
    let gfx = gfx();
    let reports: u32 = gfx.reports.on.get();
    gfx.reports.on.set(0);
    gfx.card.upload.trying.set(1);
    // Err(zero): it does not work, with the zero flag the failing step left (`stc` keeps
    // it); Ok(mode): it works, with the mode it leaves.
    let tried = (|| -> Result<Result<u32, bool>, Refused> {
        // Converted: `rc2k_core::gfx::screen_modes::look_up`.
        use rc2k_core::gfx::screen_modes::{Lookup, look_up};
        let count: u32 = display.modes.count.get();
        let table = core::mem::offset_of!(crate::gfx::modes::Modes, table);
        match look_up(display.modes.entries(table, count), mode) {
            Lookup::NoBits => return Ok(Err(true)),
            // A count of 0 gives the first flag state, else the countdown's.
            Lookup::Missing => return Ok(Err(count != 0)),
            Lookup::Found(_) => {}
        }
        // The flags at the call: the search's match (zero); `gfx::set_screen_mode` sets
        // only the carry.
        if !crate::platform::set_mode_now(mode, gfx) {
            return Ok(Err(true));
        }
        if display.mode.mode_flags.get() & 0x10_0000 == 0 {
            return Ok(Ok(mode));
        }
        let (video, _) = crate::platform_3d::allocate_textures_now(0x40, 0x40, 1, 2, 0, gfx)?;
        crate::platform_3d::release_all_textures(gfx);
        Ok(Ok(video))
    })()?;
    gfx.card.upload.trying.set(0);
    gfx.reports.on.set(reports);
    Ok(match tried {
        Ok(eax) => CarryZeroFlags {
            carry: false,
            zero: eax == 0,
        },
        Err(zero) => CarryZeroFlags { carry: true, zero },
    })
}
