//! `scene::set_up_lit_engine`: `scene::set_up_engine`, `race::find_present_courses`, the
//! light's settings (lowest 0x50, highest 0xFF, level and strength 0xB4,
//! `ObjectWork.light_turns[0]` 0x6E, `ObjectWork.light_turns[1]` 0x1E) and
//! `scene::fill_light_tables`.
//!
//! Caller: `menu::play_films`. Its calls by place: 0 `set_up_engine`, 1 `find_present_courses`,
//! 2 `fill_light_tables`. Refuses what its calls refuse.

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn set_up_lit_engine(
    race: &'static Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    crate::scene::set_up_engine::set_up_engine_in(race, scene)?;
    crate::race::find_present_courses::find_present_courses(race, scene)?;
    // The light's lowest, highest, level and strength, and the light's turns.
    let scratch = scene.scratch.front_end();
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
