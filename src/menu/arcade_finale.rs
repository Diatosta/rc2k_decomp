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
use crate::menu::owner::Menus;
use crate::menu::owner::Play;

pub fn arcade_finale(
    menus: &Menus,
    play: &Play,
    sound: &crate::sound::owner::Sound,
    race: &crate::race::owner::Race,
) -> Result<(), Refused> {
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
    // Converted: `rc2k_core::menu::arcade::levels_left`.
    race.shared
        .state
        .results
        .stage_index
        .set(rc2k_core::menu::arcade::levels_left(level));
    race.shared.state.results.finished.set(0);
    crate::race::sound_tables::set_up_race_sound_objects(
        race,
        &crate::race::sound_tables::ObjectVars::game(),
        sound,
    )?;
    crate::sound::set_up_race_sounds::set_up_race_sounds(sound)?;
    crate::menu::show_loading_picture::show_loading_picture(menus)?;
    crate::race::stage_free::render3d_tidy(race)?;
    crate::race::stage_free::objects_tidy(race)?;
    // `nullsub_6`.
    crate::race::race_end::release_textures();
    crate::drive::finish_race_0::finish_race_0(race)?;
    crate::sound::stop_music::stop_music();
    crate::platform::set_menu_mode_now();
    crate::menu::show_loading_picture::show_loading_picture(menus)?;
    crate::scene::set_up_engine::set_up_engine()?;
    crate::scene::set_up_front_end_scene::set_up_front_end_scene()?;
    crate::scene::set_up_engine::set_up_engine()?;
    crate::sound::stop_music::stop_music();
    crate::sound::play_tune_3::play_tune_3();
    // The pending key taken (eax 0).
    crate::input::take_key::take_key();
    Ok(())
}
