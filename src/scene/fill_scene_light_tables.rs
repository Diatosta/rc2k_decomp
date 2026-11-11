//! `scene::fill_scene_light_tables` (0x4433AD): the light tables from the scene's own
//! colours, whatever the switch says.
//!
//! `FrameSwitches.scene_set_up` is kept, set to 1 for `scene::fill_light_tables` (which then takes the
//! scene's colours) and put back. Two callers.

use crate::race::owner::Race;
use crate::scene::fill_light_tables::fill_light_tables;

// ---- game data ----

// ---- Rust version ----

pub fn fill_scene_light_tables(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), crate::errors::BadInput> {
    let shared = &race.shared;
    let kept: u32 = shared.switches.scene_set_up.get();
    shared.switches.scene_set_up.set(1u32);
    let done = fill_light_tables(scene, race);
    shared.switches.scene_set_up.set(kept);
    done
}
