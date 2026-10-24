//! `scene::set_up_lit_engine` (0x415515): `scene::set_up_engine`, `race::
//! find_present_courses`, the light's settings (`g_lightLowest` 0x50, `g_lightHighest`
//! 0xFF, `g_lightLevel` and `g_lightStrength` 0xB4, `dword_635094` 0x6E, `dword_635098`
//! 0x1E) and `scene::fill_light_tables`, all inside `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Callers: `menu::play_films` and `scene::rebuild_front_end`, which call the Rust version
//! directly. Its calls by place: 0 `set_up_engine`, 1 `find_present_courses`, 2
//! `fill_light_tables`. It calls in its first five bytes: a private copy (its three calls
//! relocated) with a hand-made patch.
//! Refuses what its calls refuse (`find_present_courses` then runs in the game, through
//! its hook). Shadow mode runs the original alone (`set_up_engine` sets up the cars).
//! Self-test: a copy whose calls go to stand-ins that note them, against the Rust version
//! with the same, on the game's light settings (put back).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_write, rva};

/// `g_lightLowest`, `g_lightHighest`, `g_lightLevel`, `g_lightStrength`, `dword_635094`,
/// `dword_635098` and their values.
const SETTINGS: [(usize, u32); 6] = [
    (0x635084, 0x50),
    (0x635088, 0xFF),
    (0x63508C, 0xB4),
    (0x635090, 0xB4),
    (0x635094, 0x6E),
    (0x635098, 0x1E),
];

/// The Rust version (the original keeps every register: `pusha` / `popa`).
pub fn set_up_lit_engine() -> Result<(), Refused> {
    unsafe {
        crate::scene::set_up_engine::set_up_engine()?;
        crate::race::find_present_courses::find_present_courses()?;
        for (at, v) in SETTINGS {
            game_write(rva(at), v);
        }
        crate::scene::fill_light_tables::fill_light_tables();
    }
    Ok(())
}
