//! `race::viewer_load_stage` (0x41CDC2): with a replay open (`replay_open` 1), its stage loaded
//! for the viewer: the front end's 3D tidied (`race::render3d_tidy`, `race::objects_tidy`,
//! `nullsub_6`, `race::release_textures`), the race's screen mode (`race::set_race_screen_mode`),
//! the loading picture (`menu::show_loading_picture`); from the replay's header the game mode,
//! players, cars and options (`header_mode` .. into `RaceState.settings.run_kind`, `g_onePlayer`, `RaceState.settings.car_kind_current`,
//! `RaceState.settings.car_kind_first`, `RaceState.settings.cars[0].codriver_sex`, `RaceState.settings.cars[1].codriver_sex`; `RaceState.settings.cars[0].controller`, `RaceState.settings.cars[1].controller` 0; `RaceState.settings.race_mode`
//! the players; the co-driver's door `byte_719C8C` into `RaceState.settings.cars[0].race_number` and the cars' block, logged
//! "doornr="); the race's sounds (`race::set_up_race_sound_objects`, `sound::set_up_race_sounds`);
//! the fonts unless there are some (`gfx::build_fonts`, `viewer_fonts_built` 1); the header read in
//! (`race::read_replay_header`), the stage (`race::load_stage`), the sounds reset
//! (`race::reset_race_sounds`), the page's palettes (`race::viewer_palettes`), `viewer_stage_loaded` 1.
//! Then (always) the pending key taken (`input::take_key`).
//!
//! Callers: `race::replay_viewer`, `race::viewer_action` (0x14, after another replay is opened),
//! `race::choose_save_slot`.

use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;
use crate::session::Dword;

const DOOR_TEXT: &std::ffi::CStr = c"doornr="; // "doornr="
/// (to, from): the header's settings.
fn settings(race: &Race) -> [(&Dword, &crate::session::PlainDword); 5] {
    let shared = &race.shared;
    let work = race.replay_work();
    [
        (&shared.state.settings.run_kind, &work.header_mode),
        (&shared.state.settings.car_kind_current, &work.header_kind_0),
        (&shared.state.settings.car_kind_first, &work.header_kind_1),
        (
            &shared.state.settings.cars[0].codriver_sex,
            &work.header_codriver_sex_0,
        ),
        (
            &shared.state.settings.cars[1].codriver_sex,
            &work.header_codriver_sex_1,
        ),
    ]
}

pub fn viewer_load_stage(race: &'static Race, o: Owners<'static>) -> Result<(), Refused> {
    let (gfx, sound, scene) = (o.gfx, o.sound, o.scene);
    let shared = &race.shared;
    use crate::race::{race_end, race_shell, sound_tables, stage_free, stage_loading};
    #[cfg(feature = "remote")]
    crate::remote::shots_loading(race, o.effects);
    if shared.replay_file().replay_open.is_set() {
        stage_free::render3d_tidy(race)?;
        stage_free::objects_tidy(race)?;
        // nullsub_6
        race_end::release_textures(o.scene, o.gfx);
        race_shell::set_race_screen_mode(gfx, o.system);

        crate::menu::show_loading_picture::show_loading_picture_in(o)?;
        for (to, from) in settings(race) {
            to.set(from.get());
        }
        // g_onePlayer (the sound module's) takes the header's setting too.
        sound
            .race
            .sounds
            .one_player
            .set(race.replay_work().header_one_player.get());
        shared.state.settings.cars[0].controller.set(0);
        shared.state.settings.cars[1].controller.set(0);
        shared
            .state
            .settings
            .race_mode
            .set(sound.race.sounds.one_player.get());
        // byte_719C8C: the low byte of the first dword of `unknown_7d0`.
        let door = race.replay_work().unknown_7d0[0].get() & 0xFF;
        shared.state.settings.cars[0].race_number.set(door);
        race.car_blocks().cars[0].start_slot_value.set(door);
        crate::logfile::log_write::log_nul(&DOOR_TEXT, false, 1);
        crate::logfile::log_int::log_int(door as i32, true, 1);

        sound_tables::set_up_race_sound_objects(
            race,
            &sound_tables::ObjectVars::of(sound),
            sound,
            o.audio,
            o.net,
            o.system,
        )?;
        crate::sound::set_up_race_sounds::set_up_race_sounds(o)?;
        if gfx.fonts.font_block.palettes.get() == 0 {
            crate::gfx::build_fonts::build_fonts_in(gfx, o.system, o.strings)?;
            shared.replay_file().viewer_fonts_built.set(1);
        }
        crate::race::read_replay_header::read_replay_header(race);
        { stage_loading::load_stage(race, o) }?;
        race_end::reset_race_sounds(&race_end::SoundVars::game(sound), sound, o.audio, o.system)?;
        crate::race::viewer_palettes::viewer_palettes(shared, scene, gfx, o.system)?;
        shared.replay_file().viewer_stage_loaded.set(1);
    }
    crate::input::take_key::take_key();
    Ok(())
}
