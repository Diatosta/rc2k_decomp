//! `menu::arcade_finale` (0x488155): after an arcade level is won: the finishing player is
//! player 0's +0x28, the cars' kind and a second figure take its +8, the kept value copy is cleared, the arcade race index
//! is 5 - the level, the finish flag is cleared; then the race's sound objects and sounds
//! (`race::set_up_race_sound_objects`, `sound::set_up_race_sounds`), the loading picture,
//! the 3D tidied (`render3d_tidy`, `objects_tidy`, `nullsub_6`, `release_textures`),
//! `drive::finish_race_0`, the music stopped, the menu's screen mode, the loading picture,
//! `scene::set_up_engine`, `set_up_front_end_scene`, `set_up_engine` again, the music
//! stopped, tune 3, and the pending key taken (eax 0). What `finish_race_0` shows is a
//! reading (the level's ending drive).
//!
//! The callees get eax 5 - level, ebx the level and edi player 0's record at first, then what the
//! one before left. Refuses (before anything) a missing player 0.

use crate::errors::Refused;
use crate::owners::Owners;

pub fn arcade_finale(o: Owners<'static>) -> Result<(), Refused> {
    let (menus, sound, race) = (o.menus, o.sound, o.race);
    let play = &menus.play;
    let first = play.players.players[0].get();
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
    // The logic is in `rc2k_core::menu::arcade::levels_left`.
    race.shared
        .state
        .results
        .stage_index
        .set(rc2k_core::menu::arcade::levels_left(level));
    race.shared.state.results.finished.set(0);
    crate::race::sound_tables::set_up_race_sound_objects(
        race,
        &crate::race::sound_tables::ObjectVars::of(sound),
        sound,
        o.audio,
        o.net,
        o.system,
    )?;
    crate::sound::set_up_race_sounds::set_up_race_sounds(o)?;
    crate::menu::show_loading_picture::show_loading_picture_in(o)?;
    crate::race::stage_free::render3d_tidy(race)?;
    crate::race::stage_free::objects_tidy(race)?;
    // `nullsub_6`.
    crate::race::race_end::release_textures(o.scene, o.gfx);
    crate::drive::finish_race_0::finish_race_0(race, o)?;
    crate::sound::stop_music::stop_music(o.audio, o.system);
    crate::platform::set_menu_mode_now(o.gfx, o.system);
    crate::menu::show_loading_picture::show_loading_picture_in(o)?;
    crate::scene::set_up_engine::set_up_engine_in(race, o.scene)?;
    crate::scene::set_up_front_end_scene::set_up_front_end_scene_in(o)?;
    crate::scene::set_up_engine::set_up_engine_in(race, o.scene)?;
    crate::sound::stop_music::stop_music(o.audio, o.system);
    crate::sound::play_tune_3::play_tune_3(o.sound, o.audio, o.system);
    // The pending key taken (eax 0).
    crate::input::take_key::take_key();
    Ok(())
}
