//! `scene::fill_scene_light_tables` (0x4433AD): the light tables from the scene's own
//! colours, whatever the switch says.
//!
//! In:  nothing
//! Out: as `scene::fill_light_tables` leaves them
//!
//! `dword_621480` is kept, set to 1 for `scene::fill_light_tables` (which then takes the
//! scene's colours) and put back. Two callers.

use crate::race::owner::race;
use crate::scene::fill_light_tables::fill_light_tables;

// ---- game data ----

// ---- Rust version ----

pub fn fill_scene_light_tables(
    scene: &crate::scene::owner::Scene,
) -> Result<(), crate::errors::BadInput> {
    let race = race();
    let shared = &race.shared;
    let kept: u32 = shared.switches.other_way.bits();
    shared.switches.other_way.set_bits(1u32);
    let done = fill_light_tables(scene, race);
    shared.switches.other_way.set_bits(kept);
    done
}
