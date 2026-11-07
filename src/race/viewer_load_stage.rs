//! `race::viewer_load_stage` (0x41CDC2): with a replay open (`dword_5BEC88` 1), its stage
//! loaded for the viewer: the front end's 3D tidied (`race::render3d_tidy`,
//! `race::objects_tidy`, `nullsub_6`, `race::release_textures`), the race's screen mode
//! (`race::set_race_screen_mode`), the loading picture (`menu::show_loading_picture`);
//! from the replay's header the game mode, players, cars and options (`dword_719B18` ..
//! into `dword_75ACD8`, `g_onePlayer`, `dword_75ACE8`, `dword_75ACEC`, `dword_75AE20`,
//! `dword_75AF8C`; `dword_75AE28`, `dword_75AF94` 0; `dword_75ACE0` the players; the
//! co-driver's door `byte_719C8C` into `dword_75AE1C` and the cars' block, logged
//! "doornr="); the race's sounds (`race::set_up_race_sound_objects`,
//! `sound::set_up_race_sounds`); the fonts unless there are some (`gfx::build_fonts`,
//! `dword_5BECC4` 1); the header read in (`race::read_replay_header`), the stage
//! (`race::load_stage`), the sounds reset (`race::reset_race_sounds`), the page's
//! palettes (`race::viewer_palettes`), `dword_5BECC0` 1. Then (always) the pending key
//! taken (`input::take_key`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::replay_viewer`, `race::viewer_action` (0x14, after another replay is
//! opened), `race::choose_save_slot`. The only register it reads at entry is esi, which
//! goes to `release_textures` and `set_race_screen_mode` (attrib), whose Rust versions
//! read none.

use crate::errors::Refused;
use crate::gfx::owner::gfx;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::Race;
use crate::race::owner_shared::Shared;
use crate::race::replay_work_store::REPLAY_WORK;
use crate::session::Dword;

const DOOR_TEXT: &std::ffi::CStr = c"doornr="; // "doornr="
/// (to, from): the header's settings.
fn settings(shared: &Shared) -> [(&Dword, &'static Dword); 5] {
    [
        (&shared.state.settings.run_kind, &REPLAY_WORK.header_mode),
        (
            &shared.state.settings.car_kind_current,
            &REPLAY_WORK.header_kind_0,
        ),
        (
            &shared.state.settings.car_kind_first,
            &REPLAY_WORK.header_kind_1,
        ),
        (
            &shared.state.settings.cars[0].codriver_sex,
            &REPLAY_WORK.header_codriver_sex_0,
        ),
        (
            &shared.state.settings.cars[1].codriver_sex,
            &REPLAY_WORK.header_codriver_sex_1,
        ),
    ]
}

/// The Rust version (every register kept).
pub fn viewer_load_stage(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    use crate::race::{race_end, race_shell, sound_tables, stage_free, stage_loading};
    #[cfg(feature = "remote")]
    crate::remote::shots_loading();
    if shared.replay_file.replay_open.get() == 1 {
        stage_free::render3d_tidy()?;
        stage_free::objects_tidy()?;
        // nullsub_6
        race_end::release_textures();
        race_shell::set_race_screen_mode();

        crate::menu::show_loading_picture::show_loading_picture()?;
        for (to, from) in settings(shared) {
            to.set(from.get());
        }
        // g_onePlayer (the sound module's) takes the header's setting too.
        crate::sound::owner::sound()
            .race
            .sounds
            .one_player
            .set(REPLAY_WORK.header_one_player.get());
        shared.state.settings.cars[0].controller.set(0);
        shared.state.settings.cars[1].controller.set(0);
        shared
            .state
            .settings
            .race_mode
            .set(crate::sound::owner::sound().race.sounds.one_player.get());
        // byte_719C8C: the low byte of the first dword of `unknown_7d0`.
        let door = REPLAY_WORK.unknown_7d0[0].get() & 0xFF;
        shared.state.settings.cars[0].race_number.set(door);
        CAR_BLOCKS.cars[0].start_slot_value.set(door);
        crate::logfile::log_write::log_nul(&DOOR_TEXT, false, 1);
        crate::logfile::log_int::log_int(door as i32, true, 1);

        sound_tables::set_up_race_sound_objects(&sound_tables::ObjectVars::game())?;
        crate::sound::set_up_race_sounds::set_up_race_sounds()?;
        if gfx().fonts.menu_fonts.palettes.get() == 0 {
            { crate::gfx::build_fonts::build_fonts() }?;
            shared.replay_file.viewer_fonts_built.set(1);
        }
        crate::race::read_replay_header::read_replay_header(shared);
        { stage_loading::load_stage(race) }?;
        race_end::reset_race_sounds(&race_end::SoundVars::game())?;
        crate::race::viewer_palettes::viewer_palettes(shared)?;
        shared.replay_file.viewer_stage_loaded.set(1);
    }
    crate::input::take_key::take_key();
    Ok(())
}
