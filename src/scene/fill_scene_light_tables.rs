//! `scene::fill_scene_light_tables`: the light tables from the scene's own colours, whatever the
//! switch says: `FrameSwitches.scene_set_up` is kept, set to 1 for `scene::fill_light_tables`
//! (which then takes the scene's colours) and put back.

use crate::race::owner::Race;
use crate::scene::fill_light_tables::fill_light_tables;

// ---- game data ----

pub fn fill_scene_light_tables(
    race: &Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), crate::errors::BadInput> {
    let shared = &race.shared;
    let kept: u32 = shared.switches.borrow().scene_set_up;
    shared.switches.borrow_mut().scene_set_up = 1u32;
    let done = fill_light_tables(scene, race);
    shared.switches.borrow_mut().scene_set_up = kept;
    done
}
