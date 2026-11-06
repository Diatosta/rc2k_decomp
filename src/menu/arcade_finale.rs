//! `menu::arcade_finale` (0x488155): after an arcade level is won: `dword_75B100` player
//! 0's +0x28, `dword_75B104` and `dword_75ACE8` its +8, `dword_75ACE4` 0, `dword_75B0FC`
//! 5 - the level, `dword_75B108` 0; then the race's sound objects and sounds
//! (`race::set_up_race_sound_objects`, `sound::set_up_race_sounds`), the loading picture,
//! the 3D tidied (`render3d_tidy`, `objects_tidy`, `nullsub_6`, `release_textures`),
//! `drive::finish_race_0`, the music stopped, the menu's screen mode, the loading picture,
//! `scene::set_up_engine`, `set_up_front_end_scene`, `set_up_engine` again, the music
//! stopped, tune 3, and the pending key taken (eax 0). What `finish_race_0` shows is a
//! reading (the level's ending drive).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! The callees get eax 5 - level, ebx the level and edi player 0's record at first, then what the
//! one before left. Refuses (before anything) a missing player 0.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::race::state_store::RACE_STATE;

/// The Rust version. The original keeps every register of its caller.
pub fn arcade_finale() -> Result<(), Refused> {
    let play = &menus().play;
    let first = play.players.players[0].get();
    RACE_STATE.results.finish_player.set(first.race_number);
    RACE_STATE.results.finish_kind.set(first.car_number);
    RACE_STATE.settings.car_kind_current.set(first.car_number);
    RACE_STATE.settings.kept.set(0);
    let level = play.arcade.level.get();
    // Converted: `rc2k_core::menu::arcade::levels_left`.
    RACE_STATE
        .results
        .stage_index
        .set(rc2k_core::menu::arcade::levels_left(level));
    RACE_STATE.results.finished.set(0);
    crate::race::sound_tables::set_up_race_sound_objects(
        &crate::race::sound_tables::ObjectVars::game(),
    )?;
    crate::sound::set_up_race_sounds::set_up_race_sounds()?;
    crate::menu::show_loading_picture::show_loading_picture()?;
    crate::race::stage_free::render3d_tidy()?;
    crate::race::stage_free::objects_tidy()?;
    // `nullsub_6`.
    crate::race::race_end::release_textures();
    crate::drive::finish_race_0::finish_race_0()?;
    crate::sound::stop_music::stop_music();
    crate::platform::set_menu_mode_now();
    crate::menu::show_loading_picture::show_loading_picture()?;
    crate::scene::set_up_engine::set_up_engine()?;
    crate::scene::set_up_front_end_scene::set_up_front_end_scene()?;
    crate::scene::set_up_engine::set_up_engine()?;
    crate::sound::stop_music::stop_music();
    crate::sound::play_tune_3::play_tune_3();
    // The pending key taken (eax 0).
    crate::input::take_key::take_key();
    Ok(())
}
