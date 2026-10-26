//! `gfx::keep_race_screen_mode` (0x404805): mode eax kept as the race's
//! (`g_raceScreenMode`), and its pixel kind (the second byte) with 0x320002 (800 x 600 by
//! its size bits, the menus' other bits) tried (`gfx::try_screen_mode`): if that works it
//! becomes the menus' mode (`g_menuScreenMode`).
//!
//! In:  eax = the mode
//! Out: every register kept (`pusha` .. `popa`); the flags as `try_screen_mode` leaves them
//!
//! One caller, `gfx::screen_mode_choices`. Its call through `part` (0; in both programs
//! `try_screen_mode`'s Rust version). Shadow mode runs the original alone (the trial sets
//! the screen). Self-test: a copy whose call goes to a stand-in (working or not), against
//! the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::viewer::{Part, put, regs_entry};

pub const RACE_MODE: usize = 0x4F4948; // g_raceScreenMode
pub const MENU_MODE: usize = 0x4F4944; // g_menuScreenMode

/// The Rust version: the registers and flags it leaves.
pub fn keep_race_screen_mode(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    put(RACE_MODE, entry.eax);
    // Converted: `rc2k_core::gfx::screen_modes::menu_mode_for`.
    let mut r = Regs {
        eax: rc2k_core::gfx::screen_modes::menu_mode_for(entry.eax),
        ..*entry
    };
    part(0, &mut r)?;
    if !r.carry() {
        put(MENU_MODE, r.eax);
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// In both programs: `try_screen_mode`'s Rust version (its calls through their hooks).
pub fn game_part(_place: u32, r: &mut Regs) -> Result<(), Refused> {
    *r = crate::gfx::try_screen_mode::try_screen_mode(
        r,
        &mut crate::gfx::try_screen_mode::game_part,
    )?;
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| keep_race_screen_mode(r, &mut game_part))
}
