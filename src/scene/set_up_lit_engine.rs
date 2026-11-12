//! `scene::set_up_lit_engine` (0x415515): `scene::set_up_engine`, `race::
//! find_present_courses`, the light's settings (`g_lightLowest` 0x50, `g_lightHighest`
//! 0xFF, `g_lightLevel` and `g_lightStrength` 0xB4, `ObjectWork.light_turns[0]` 0x6E, `ObjectWork.light_turns[1]`
//! 0x1E) and `scene::fill_light_tables`.
//!
//! Caller: `menu::play_films` (the original's `scene::rebuild_front_end`, 0x4154CD, too:
//! only a lobby's launch reaches it, left out with the lobbies, `net::open_lobby`). Its calls by place: 0 `set_up_engine`, 1 `find_present_courses`, 2
//! `fill_light_tables`. Refuses what its calls refuse.

use crate::errors::Refused;
use crate::race::owner::Race;

/// The Rust version (the original keeps every register: `pusha` / `popa`).
pub fn set_up_lit_engine(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    crate::scene::set_up_engine::set_up_engine_in(race, scene)?;
    crate::race::find_present_courses::find_present_courses(race, scene)?;
    // `g_lightLowest`, `g_lightHighest`, `g_lightLevel`, `g_lightStrength`,
    // `ObjectWork.light_turns[0]`, `ObjectWork.light_turns[1]`.
    let scratch = crate::race::view_scratch::front_end();
    let light = &scratch.light;
    light.light_lowest.set(0x50u32);
    light.light_highest.set(0xFFu32);
    light.light_level.set(0xB4u32);
    scratch.object.light_strength.set(0xB4u32);
    light.light_turns[0].set(0x6Eu32);
    light.light_turns[1].set(0x1Eu32);
    crate::scene::fill_light_tables::fill_light_tables(scene, race)?;
    Ok(())
}
