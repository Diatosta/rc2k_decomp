//! `menu::arcade_finale`: after an arcade level is won. The finishing player is player 0's +0x28,
//! the cars' kind and a second figure take its +8, the kept value copy is cleared, the arcade
//! race index becomes 5 - the level and the finish flag is cleared. Then come the race's sound
//! objects and sounds (`race::set_up_race_sound_objects`, `sound::set_up_race_sounds`), the
//! loading picture, the 3D tidied (`render3d_tidy`, `objects_tidy`, `release_textures`),
//! `drive::finish_race_0`, the music stopped, the menu's screen mode, the loading picture,
//! `scene::set_up_engine`, `set_up_front_end_scene`, `set_up_engine` again, the music stopped,
//! tune 3, and the pending key taken. What `finish_race_0` shows is a reading (the level's
//! ending drive).
//!
//! Refuses (before anything) a missing player 0.

use crate::errors::Refused;
use crate::owners::Owners;

pub fn arcade_finale(o: Owners<'static>) -> Result<(), Refused> {
    let (menus, sound, race) = (o.menus, o.sound, o.race);
    let play = &menus.play;
    let first = play.slots.records[0].get();
    race.shared
        .state
        .results
        .finish_player
        .set(first.race_number);
    race.shared.state.results.finish_kind.set(first.car_number);
    race.shared
        .state
        .settings
        .car_kind_current
        .set(first.car_number);
    race.shared.state.settings.kept.set(0);
    let level = play.arcade.level.get();
    race.shared
        .state
        .results
        .stage_index
        .set(rc2k_core::menu::arcade::levels_left(level));
    race.shared.state.results.finished.set(0);
    crate::race::sound_tables::set_up_race_sound_objects(
        o,
        &crate::race::sound_tables::ObjectVars::of(sound),
    )?;
    crate::sound::set_up_race_sounds::set_up_race_sounds(o)?;
    crate::menu::show_loading_picture::show_loading_picture_in(o)?;
    crate::race::stage_free::render3d_tidy(race, &o.scene.scratch)?;
    crate::race::stage_free::objects_tidy(race, &o.scene.scratch)?;
    crate::race::race_end::release_textures(o.scene, o.gfx);
    crate::drive::finish_race_0::finish_race_0(race, o)?;
    crate::sound::stop_music::stop_music(o);
    crate::platform::set_menu_mode_now(o.gfx, o.system);
    crate::menu::show_loading_picture::show_loading_picture_in(o)?;
    crate::scene::set_up_engine::set_up_engine_in(race, o.scene)?;
    crate::scene::set_up_front_end_scene::set_up_front_end_scene_in(o)?;
    crate::scene::set_up_engine::set_up_engine_in(race, o.scene)?;
    crate::sound::stop_music::stop_music(o);
    crate::sound::play_tune_3::play_tune_3(o);
    // The pending key is taken and dropped.
    crate::input::take_key::take_key(o.input);
    Ok(())
}
