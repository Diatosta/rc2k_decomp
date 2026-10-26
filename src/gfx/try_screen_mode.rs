//! `gfx::try_screen_mode` (0x404AED): whether screen mode eax works: the mode's entry
//! looked up in the mode table (`g_screenModeTable`, `g_screenModeCount`, by its bits but
//! the third byte's low nibble's: `& 0xF0FFFF`, 0 failing), the mode set
//! (`gfx::set_screen_mode`), and with `g_screenModeFlags` bit 0x100000 a trial of the
//! textures (`gfx::allocate_textures` with eax 0x40, ebx 0x40, ecx 1, edx 2, ebp 0, then
//! `gfx::release_all_textures`). Meanwhile `dword_5ABA88` is 1 and the reports
//! (`dword_4F52AC`) off, both put back after.
//!
//! In:  eax = the mode
//! Out: carry = the mode does not work; else carry clear and the zero flag by the eax the
//!      last call left; every register kept (`pusha` .. `popa`)
//!
//! Callers: `gfx::choose_screen_modes` (its list of modes), `gfx::screen_mode_choices`,
//! `gfx::keep_race_screen_mode`, and two in the race's set-up (0x442D2C, 0x4432B6). Its
//! three calls (set the mode, allocate, release: the platform's) are direct (2026-10-23).

use crate::errors::Refused;
use crate::framework::Flags;
use crate::gfx::modes::MODES;
use crate::gfx::reports::REPORTS;
use crate::gfx::screen_state::SCREEN_MODE;
use crate::gfx::upload::UPLOAD;

// ---- Rust version ----

/// The Rust version on `mode`: the flags it leaves (carry: it does not work).
pub fn try_screen_mode(mode: u32) -> Result<Flags, Refused> {
    let reports: u32 = REPORTS.on.get();
    REPORTS.on.set(0);
    UPLOAD.trying.set(1);
    // Err(zero): it does not work, with the zero flag the failing step left (`stc` keeps
    // it); Ok(eax): it works, with the eax it leaves.
    let tried = (|| -> Result<Result<u32, bool>, Refused> {
        // Converted: `rc2k_core::gfx::screen_modes::look_up`.
        use rc2k_core::gfx::screen_modes::{Lookup, look_up};
        let count: u32 = MODES.count.get();
        let table = MODES.table_addr() as u32;
        match look_up(crate::system::find_entry::entries(table, count), mode) {
            Lookup::NoBits => return Ok(Err(true)),
            // `cmp ecx, 1` on a count of 0, else the `dec ecx` that reached 0.
            Lookup::Missing => return Ok(Err(count != 0)),
            Lookup::Found(_) => {}
        }
        // The flags at the call: the search's match (zero); `gfx::set_screen_mode` sets
        // only the carry.
        if !crate::platform::set_mode_now(mode) {
            return Ok(Err(true));
        }
        if SCREEN_MODE.mode_flags.get() & 0x10_0000 == 0 {
            return Ok(Ok(mode));
        }
        let (video, _) = crate::platform_3d::allocate_textures_now(0x40, 0x40, 1, 2, 0)?;
        crate::platform_3d::release_all_textures();
        Ok(Ok(video))
    })()?;
    UPLOAD.trying.set(0);
    REPORTS.on.set(reports);
    Ok(match tried {
        Ok(eax) => Flags {
            carry: false,
            zero: eax == 0,
        },
        Err(zero) => Flags { carry: true, zero },
    })
}
