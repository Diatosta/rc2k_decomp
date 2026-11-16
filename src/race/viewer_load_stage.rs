//! `race::viewer_load_stage`: with a replay open (`replay_open` 1), loads its stage for the
//! viewer. The front end's 3D is tidied (`race::render3d_tidy`, `race::objects_tidy`,
//! `race::release_textures`), the race's screen mode is set (`race::set_race_screen_mode`) and
//! the loading picture shown (`menu::show_loading_picture`). From the replay's header come the
//! game mode, players, cars and options (`header_mode` and the words after it go into
//! `RaceState.settings.run_kind`, `one_player`, `car_kind_current`, `car_kind_first` and both
//! cars' `codriver_sex`; both cars' `controller` are set to 0 and `race_mode` to the players'
//! count; the co-driver's door goes into `RaceState.settings.cars[0].race_number` and the cars'
//! block, logged as "doornr="). Then the race's sounds are set up
//! (`race::set_up_race_sound_objects`, `sound::set_up_race_sounds`), the fonts built unless there
//! are some (`gfx::build_fonts`, `viewer_fonts_built` 1), the header read in
//! (`race::read_replay_header`), the stage loaded (`race::load_stage`), the sounds reset
//! (`race::reset_race_sounds`), the page's palettes made (`race::viewer_palettes`) and
//! `viewer_stage_loaded` set to 1. In every case the pending key is taken last
//! (`input::take_key`).

use crate::errors::Refused;
use crate::memory::session::DwordLike;
use crate::owners::Owners;
use crate::race::owner::Race;

const DOOR_TEXT: &std::ffi::CStr = c"doornr=";

/// The header's settings as (to, from).
fn settings(race: &Race) -> [(&dyn DwordLike, &crate::memory::session::PlainDword); 5] {
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
    crate::remote::shots_loading(race, o.race.effects.get(), &o.system.session.picture_clock);
    if shared.replay_file().replay_open.is_set() {
        stage_free::render3d_tidy(race, &scene.scratch)?;
        stage_free::objects_tidy(race, &scene.scratch)?;
        race_end::release_textures(o.scene, o.gfx);
        race_shell::set_race_screen_mode(gfx, o.system);

        crate::menu::show_loading_picture::show_loading_picture_in(o)?;
        for (to, from) in settings(race) {
            to.set(from.get());
        }
        // The sound module's `one_player` takes the header's setting too.
        sound
            .threads
            .shared
            .one_player
            .set(race.replay_work().header_one_player.get());
        shared.state.settings.cars[0].controls.controller.set(0);
        shared.state.settings.cars[1].controls.controller.set(0);
        shared
            .state
            .settings
            .race_mode
            .set(sound.threads.shared.one_player.get());
        // The door number is the low byte of the first dword of `unknown_7d0`.
        let door = race.replay_work().unknown_7d0[0].get() & 0xFF;
        shared.state.settings.cars[0].race_number.set(door);
        race.car_blocks().cars[0].start_slot_value.set(door);
        crate::logfile::log_write::log_nul(&DOOR_TEXT, false, 1);
        crate::logfile::log_int::log_int(door as i32, true, 1);

        sound_tables::set_up_race_sound_objects(o, &sound_tables::ObjectVars::of(sound))?;
        crate::sound::set_up_race_sounds::set_up_race_sounds(o)?;
        if gfx.fonts.font_block.palettes.get() == 0 {
            crate::gfx::build_fonts::build_fonts_in(gfx, o.system, o.strings)?;
            shared.replay_file().viewer_fonts_built.set(1);
        }
        crate::race::read_replay_header::read_replay_header(race);
        { stage_loading::load_stage(race, o) }?;
        race_end::reset_race_sounds(&race_end::SoundVars::game(sound), o)?;
        crate::race::viewer_palettes::viewer_palettes(shared, scene, gfx, o.system)?;
        shared.replay_file().viewer_stage_loaded.set(1);
    }
    crate::input::take_key::take_key(o.input);
    Ok(())
}
