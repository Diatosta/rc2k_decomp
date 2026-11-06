//! `scene::set_up_lit_engine` (0x415515): `scene::set_up_engine`, `race::
//! find_present_courses`, the light's settings (`g_lightLowest` 0x50, `g_lightHighest`
//! 0xFF, `g_lightLevel` and `g_lightStrength` 0xB4, `dword_635094` 0x6E, `dword_635098`
//! 0x1E) and `scene::fill_light_tables`, all inside `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `menu::play_films` (the original's `scene::rebuild_front_end`, 0x4154CD, too:
//! only a lobby's launch reaches it, left out with the lobbies, `net::open_lobby`). Its calls by place: 0 `set_up_engine`, 1 `find_present_courses`, 2
//! `fill_light_tables`. Refuses what its calls refuse.

use crate::errors::Refused;
use crate::race::owner::race;
use crate::scene::object_store::OBJECT_WORK;

/// The Rust version (the original keeps every register: `pusha` / `popa`).
pub fn set_up_lit_engine() -> Result<(), Refused> {
    let course = &race().course;
    crate::scene::set_up_engine::set_up_engine()?;
    crate::race::find_present_courses::find_present_courses()?;
    // `g_lightLowest`, `g_lightHighest`, `g_lightLevel`, `g_lightStrength`,
    // `dword_635094`, `dword_635098`.
    OBJECT_WORK.light_lowest.set_bits(0x50u32);
    OBJECT_WORK.light_highest.set_bits(0xFFu32);
    OBJECT_WORK.light_level.set_bits(0xB4u32);
    OBJECT_WORK.light_strength.set_bits(0xB4u32);
    OBJECT_WORK.light_turns[0].set_bits(0x6Eu32);
    OBJECT_WORK.light_turns[1].set_bits(0x1Eu32);
    crate::scene::fill_light_tables::fill_light_tables(crate::scene::owner::scene(), course)?;
    Ok(())
}
