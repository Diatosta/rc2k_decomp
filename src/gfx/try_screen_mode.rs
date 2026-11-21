//! Whether the given screen mode works: the mode's entry looked up in the mode table (by its bits
//! but the third byte's low nibble; 0 fails), the mode set (`gfx::set_screen_mode`), and with the
//! mode flags' bit 0x100000 a trial of the textures (`gfx::allocate_textures` for 0x40 x 0x40
//! textures, then `gfx::release_all_textures`). Meanwhile `trying` is 1 and the reports
//! (`reports.on`) off, both put back after. Its three calls (set the mode, allocate, release: the
//! platform's) are direct.

use crate::errors::Refused;
use crate::gfx::owner::Gfx;

/// Tries `mode`: whether it works.
pub(super) fn try_screen_mode(
    gfx: &Gfx,
    system: &crate::system::owner::System,
    d3d: &crate::d3d::owner::D3d,
    mode: u32,
) -> Result<bool, Refused> {
    let display = &gfx.display;
    let reports: u32 = gfx.reports.on.get();
    gfx.reports.on.set(0);
    gfx.card.upload.trying.set(1);
    let works = (|| -> Result<bool, Refused> {
        use rc2k_core::gfx::screen_modes::{Lookup, look_up};
        let count: u32 = display.modes.count.get();
        match look_up(
            display
                .modes
                .entries(crate::gfx::modes::Modes::TABLE_AT, count),
            mode,
        ) {
            Lookup::NoBits | Lookup::Missing => return Ok(false),
            Lookup::Found(_) => {}
        }
        if !crate::platform::set_mode_now(mode, gfx, system) {
            return Ok(false);
        }
        if display.mode.mode_flags.get() & 0x10_0000 == 0 {
            return Ok(true);
        }
        crate::d3d::device::allocate_textures_now(0x40, 0x40, 1, 2, 0, gfx, d3d)?;
        crate::d3d::device::release_all_textures(gfx);
        Ok(true)
    })()?;
    gfx.card.upload.trying.set(0);
    gfx.reports.on.set(reports);
    Ok(works)
}
