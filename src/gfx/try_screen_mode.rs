//! Whether the given screen mode works: the mode's entry looked up in the mode table (by its bits
//! but the third byte's low nibble; 0 fails), the mode set (`gfx::set_screen_mode`), and with the
//! mode flags' bit 0x100000 a trial of the textures (`gfx::allocate_textures` for 0x40 x 0x40
//! textures, then `gfx::release_all_textures`). Meanwhile `trying` is 1 and the reports
//! (`reports.on`) off, both put back after. Its three calls (set the mode, allocate, release: the
//! platform's) are direct.

use crate::errors::CarryZeroFlags;
use crate::errors::Refused;
use crate::gfx::owner::Gfx;

/// Tries `mode`: the carry and zero flags it leaves (carry: it does not work).
pub fn try_screen_mode(
    gfx: &Gfx,
    system: &crate::system::owner::System,
    d3d: &crate::d3d::owner::D3d,
    mode: u32,
) -> Result<CarryZeroFlags, Refused> {
    let display = &gfx.display;
    let reports: u32 = gfx.reports.on.get();
    gfx.reports.on.set(0);
    gfx.card.upload.trying.set(1);
    // Err(zero): it does not work, with the zero flag the failing step left;
    // Ok(mode): it works, with the mode it leaves.
    let tried = (|| -> Result<Result<u32, bool>, Refused> {
        use rc2k_core::gfx::screen_modes::{Lookup, look_up};
        let count: u32 = display.modes.count.get();
        match look_up(
            display
                .modes
                .entries(crate::gfx::modes::Modes::TABLE_AT, count),
            mode,
        ) {
            Lookup::NoBits => return Ok(Err(true)),
            // A count of 0 gives the first flag state, else the countdown's.
            Lookup::Missing => return Ok(Err(count != 0)),
            Lookup::Found(_) => {}
        }
        // The zero flag is the search's match; `gfx::set_screen_mode` changes only the carry.
        if !crate::platform::set_mode_now(mode, gfx, system) {
            return Ok(Err(true));
        }
        if display.mode.mode_flags.get() & 0x10_0000 == 0 {
            return Ok(Ok(mode));
        }
        let (video, _) =
            crate::platform::direct3d::allocate_textures_now(0x40, 0x40, 1, 2, 0, gfx, d3d)?;
        crate::platform::direct3d::release_all_textures(gfx);
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
