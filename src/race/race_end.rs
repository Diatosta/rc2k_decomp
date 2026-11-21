//! The race's end and tidying (`race::tidy_everything` "SHAUNTIDYEVERYTHING", `race::tidy_stage`
//! "LOADLEVEL_TIDY"), and the replay header's record (`race::record_replay_header`).

use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::replay_work_store::ReplayWork;
use rc2k_core::audio::commands::Command;

/// Where a header setting lives in the race's variables (its first place).
pub(super) fn setting_cell(
    race: &Race,
    s: rc2k_core::race::replay::Setting,
) -> &dyn crate::memory::session::DwordLike {
    let shared = &race.shared;
    use rc2k_core::race::replay::Setting::*;
    match s {
        Seed => &shared.tables().noise_seed,
        StartPlace => &shared.tables().start_place,
        ArcadeCars => &shared.tables().arcade_cars,
        Stage => &shared.vars.get().stage_number,
        Mode => &shared.vars.get().run_kind,
        Players => &shared.view.get().players,
        Cars => &shared.view.get().race_cars,
        Kind(n) => &race.car_blocks().cars[n as usize].car_kind,
        GameOver(n) => &race.car_blocks().cars[n as usize].stop_state,
    }
}

/// Where a header setting lives in the replay's car header (from `header_stage` on).
fn header_field(work: &mut ReplayWork, s: rc2k_core::race::replay::Setting) -> &mut u32 {
    use rc2k_core::race::replay::Setting::*;
    match s {
        Stage => &mut work.header_stage,
        Mode => &mut work.header_mode,
        Players => &mut work.header_players,
        Cars => &mut work.header_cars,
        Kind(0) => &mut work.header_kind_0,
        Kind(1) => &mut work.header_kind_1,
        Kind(2) => &mut work.header_kind_2,
        Kind(3) => &mut work.header_kind_3,
        StartPlace => &mut work.header_start_place,
        ArcadeCars => &mut work.header_arcade_cars,
        Seed => &mut work.header_seed,
        GameOver(0) => &mut work.header_game_over_0,
        GameOver(1) => &mut work.header_game_over_1,
        Kind(_) | GameOver(_) => unreachable!("the header holds four kinds and two flags"),
    }
}

/// The header setting `s`'s value.
pub(super) fn header_word(race: &Race, s: rc2k_core::race::replay::Setting) -> u32 {
    *header_field(&mut race.replay_work_mut(), s)
}

/// "[replayheader.arcrandom]" .. "[replayheader.gameoverflag2]", logged in the order of
/// `HEADER_SETTINGS`.
const REPLAY_WORDS: [&std::ffi::CStr; 13] = [
    c"[replayheader.arcrandom]",
    c"[replayheader.arcstartpos]",
    c"[replayheader.numarccars]",
    c"[replayheader.track]",
    c"[replayheader.gmode]",
    c"[replayheader.numpl]",
    c"[replayheader.numcars]",
    c"[replayheader.car1]",
    c"[replayheader.car2]",
    c"[replayheader.car3]",
    c"[replayheader.car4]",
    c"[replayheader.gameoverflag1]",
    c"[replayheader.gameoverflag2]",
];
/// Copies the race's settings (the arcade's seed, start place and count, the stage, the mode, the
/// players, the cars, the first four kinds, the two players' absences) into the car header (from
/// `header_stage` on), each logged as "[replayheader.x]" followed by its value.
pub fn record_replay_header(race: &Race) {
    // Logic in `rc2k_core::race::replay::HEADER_SETTINGS`.
    use rc2k_core::race::replay::HEADER_SETTINGS;
    for s in HEADER_SETTINGS {
        let value = setting_cell(race, s).get();
        *header_field(&mut race.replay_work_mut(), s) = value;
    }
    for (n, &s) in HEADER_SETTINGS.iter().enumerate() {
        crate::logfile::log_write::log_nul(&REPLAY_WORDS[n], false, 1);
        crate::logfile::log_int::log_int(header_word(race, s) as i32, true, 1);
    }
}

/// An object's variables reset for a race: the six timed slots' start and playing flags and the
/// other slots', the hit and cue sounds none (-2), the cue queue's place none, the object not in
/// use, the revs, throttle, speed and the wheels' first shares 0.
fn reset_object(o: &crate::sound::store::SoundObjectCell) {
    let mut guard = o.borrow_mut();
    let o = &mut *guard;
    for slot in &mut o.slots {
        slot.start = 0;
        slot.playing = 0;
    }
    for slot in [&mut o.gear_slot, &mut o.slot_b, &mut o.slot_c] {
        slot.start = 0;
        slot.playing = 0;
    }
    o.hit_sound = 0xFFFF_FFFE;
    o.cue_sound = 0xFFFF_FFFE;
    o.cue_queue.count = 0;
    o.cue_queue.place = 0xFFFF_FFFF;
    o.in_use = 0;
    o.revs = 0;
    o.throttle = 0;
    o.speed = 0.0;
    for voice in &mut o.voices {
        voice.share_a = 0.0;
    }
}

/// The race sound stage's flags (the run of dwords in the stage block): all set to 0.
fn clear_stage_flags(sound: &crate::sound::owner::Sound) {
    let mut s = sound.race.stage.get().borrow_mut();
    s.once = [0; 3];
    s.timed_arm = 0;
    s.timed_waiting = 0;
    s.loop_volume_a = 0;
    s.loop_volume_b = 0;
    s.random = [0; 2];
    let mut slot = sound.race.timed_slot.get().borrow_mut();
    slot.start = 0;
    slot.playing = 0;
}

/// Waits until the sound command queue has run out.
pub fn wait_for_commands(o: crate::owners::Owners<'static>) {
    crate::sound::wait_for_commands::wait_in_game(o);
}

/// Resets the race's sounds: waits for the sound commands, sends command 7 and waits again, holds
/// the sounds back and starts the race clock; resets the eight race sound objects' 32 fields
/// and eleven variables to 0; sets the 0x48 voices' first dword to 0x1888FFFF and
/// their second status dword to -2; clears two more sound flags, sends
/// command 2 with the value 3 and clears one more flag.
pub fn reset_race_sounds(
    v: &SoundVars<'_>,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    let (sound, audio) = (o.sound, o.audio);
    wait_for_commands(o);
    crate::sound::queue::send_command_in(audio, Command::StopAll);
    crate::sound::queue::send_with_eax_in(audio, Command::EndOfTurn, Some(7));
    wait_for_commands(o);
    crate::sound::hold_sounds_back::hold_sounds_back(audio);
    crate::sound::race_clock::start_race_clock(
        crate::clock::tick_count::tick_count(&o.system.session.picture_clock),
        sound,
    );
    for o in sound.race.objects.get() {
        reset_object(o);
    }
    clear_stage_flags(sound);
    for voice in v.voices.borrow_mut().iter_mut() {
        voice.id = crate::sound::spot_voices::SpotVoice::FREE_ID;
        voice.sound = 0xFFFF_FFFE;
    }
    sound.race.stage.get().borrow_mut().shared_count = 0;
    crate::sound::queue::send_command_in(audio, Command::Room(3));
    sound.race.clock.get().borrow_mut().started = 0;
    Ok(())
}

/// Where `reset_race_sounds` finds the voices (the game's, or a test's).
pub struct SoundVars<'a> {
    pub voices: &'a std::cell::RefCell<[crate::sound::spot_voices::SpotVoice; 72]>,
}

impl<'a> SoundVars<'a> {
    pub fn game(sound: &'a crate::sound::owner::Sound) -> Self {
        Self {
            voices: &sound.race.spot_voices.get().voices,
        }
    }
}

// ---- the tidying ----

/// "LOADLEVEL_TIDY : tidymud" .. "finis".
const STAGE_WORDS: [&std::ffi::CStr; 10] = [
    c"LOADLEVEL_TIDY : tidymud",
    c"LOADLEVEL_TIDY : render3d_tidy",
    c"LOADLEVEL_TIDY : world3d_tidy",
    c"LOADLEVEL_TIDY : unload game fonts",
    c"LOADLEVEL_TIDY : unload game map",
    c"LOADLEVEL_TIDY : close textures",
    c"LOADLEVEL_TIDY : dealloc course 2",
    c"LOADLEVEL_TIDY : dealloc course",
    c"LOADLEVEL_TIDY : objects tidy",
    c"LOADLEVEL_TIDY : finis",
];
/// "-----", "SHAUNTIDYEVERYTHING : START" .. "finis", "-----".
const EVERYTHING_WORDS: [&std::ffi::CStr; 8] = [
    c"---------------------------------",
    c"SHAUNTIDYEVERYTHING : START",
    c"SHAUNTIDYEVERYTHING : destroy controls",
    c"SHAUNTIDYEVERYTHING : gamemodestuff tidy",
    c"SHAUNTIDYEVERYTHING : loadlevel tidy",
    c"SHAUNTIDYEVERYTHING : stop sounds",
    c"SHAUNTIDYEVERYTHING : finis",
    c"---------------------------------",
];

/// The tidying's steps. `forgiving` is for a race that stopped half way: a step that fails
/// (what it frees was never made) is logged and the others still run.
#[derive(Clone, Copy)]
struct Tidy {
    forgiving: bool,
}

impl Tidy {
    /// A step's result: kept as it is, or, when forgiving, logged and let go.
    fn forgive(&self, result: Result<(), Refused>) -> Result<(), Refused> {
        match result {
            Err(why) if self.forgiving => {
                crate::program_log::log(&format!("recover: a tidying step failed: {why:?}"));
                Ok(())
            }
            other => other,
        }
    }

    /// Logs a word and a line break.
    fn log(&self, word: &std::ffi::CStr) {
        crate::logfile::log_write::log_nul(&word, true, 1);
    }

    /// Logs an empty line.
    fn newline(&self) {
        self.log(c"");
    }

    fn indent(&self, by: i32) {
        crate::logfile::write_line::state::INDENT
            .atomic()
            .fetch_add(by as u32, std::sync::atomic::Ordering::Relaxed);
    }

    /// The word, the indent in, the call, the indent out.
    fn step(
        &self,
        word: &std::ffi::CStr,
        call: impl FnOnce() -> Result<(), Refused>,
    ) -> Result<(), Refused> {
        self.log(word);
        self.indent(3);
        let result = self.forgive(call());
        self.indent(-3);
        result
    }
}

/// Tidies the stage ("LOADLEVEL_TIDY"), each step with its word and the log indented round it: the
/// mud, the point records, the marks' tidy (the log off), the fonts, the map, the textures, the
/// course's two memories, the objects (then `in_race` 0, `view_scale` `kept_scale`); "finis".
fn tidy_stage(
    s: Tidy,
    race: &Race,
    scene: &crate::scene::owner::Scene,
    effects: &Effects,
    gfx: &crate::gfx::owner::Gfx,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    use crate::race::stage_free::{
        COURSE, COURSE_2, MUD, OBJECT_POINTS, POINT_RECORDS, free, free_game_fonts,
    };
    s.step(STAGE_WORDS[0], || free(race, &scene.scratch, &MUD))?;
    s.step(STAGE_WORDS[1], || {
        free(race, &scene.scratch, &POINT_RECORDS)
    })?;
    s.log(STAGE_WORDS[2]);
    s.indent(3);
    let enabled = crate::logfile::write_line::state::ENABLED
        .atomic()
        .swap(0, std::sync::atomic::Ordering::Relaxed);
    let marks = crate::effects::tidy_marks::tidy_marks(effects);
    crate::logfile::write_line::state::ENABLED.set(enabled);
    s.indent(-3);
    s.forgive(marks)?;
    s.step(STAGE_WORDS[3], || free_game_fonts(gfx))?;
    s.step(STAGE_WORDS[4], || {
        crate::race::game_map::free_game_map(race, gfx)
    })?;
    s.step(STAGE_WORDS[5], || {
        crate::race::race_shell::release_stage_textures(gfx);
        Ok(())
    })?;
    s.step(STAGE_WORDS[6], || free(race, &scene.scratch, &COURSE_2))?;
    s.step(STAGE_WORDS[7], || free(race, &scene.scratch, &COURSE))?;
    s.log(STAGE_WORDS[8]);
    s.indent(3);
    s.forgive(free(race, &scene.scratch, &OBJECT_POINTS))?;
    race.course.number_vars.get().in_race.set(false);
    crate::scene::workers::set_game_part_racing(&scene.scratch, false);
    audio.racing.set(false);
    race.frame
        .view_and_records
        .get()
        .view_vars
        .view_scale
        .set(race.shared.vars.get().kept_scale.get());
    s.indent(-3);
    s.log(STAGE_WORDS[9]);
    Ok(())
}

/// Tidies everything ("SHAUNTIDYEVERYTHING"): a line break, a rule, "START" (an empty indent), then
/// each step with its word: the controls, the game mode, `tidy_stage`, the sounds (then `view_on`
/// and the second view's flag 1); "finis", a rule, a line break.
pub fn tidy_everything(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    tidy_everything_as(Tidy { forgiving: false }, race, o)
}

fn tidy_everything_as(
    s: Tidy,
    race: &Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    let (input, sound) = (o.input, o.sound);
    let shared = &race.shared;
    s.newline();
    s.log(EVERYTHING_WORDS[0]);
    s.log(EVERYTHING_WORDS[1]);
    s.indent(3);
    s.indent(-3);
    s.step(EVERYTHING_WORDS[2], || {
        crate::race::race_shell::release_controls(input);
        Ok(())
    })?;
    s.step(EVERYTHING_WORDS[3], || Ok(()))?;
    s.step(EVERYTHING_WORDS[4], || {
        tidy_stage(s, race, o.scene, o.race.effects.get(), o.gfx, o.audio)
    })?;
    crate::remote::course_hashes(race, o.scene, "free");
    s.log(EVERYTHING_WORDS[5]);
    s.indent(3);
    reset_race_sounds(&SoundVars::game(sound), o)?;
    shared.view.get().view_on[0].set(1u32);
    shared.view.get().view_on[1].set(1u32);
    s.indent(-3);
    s.log(EVERYTHING_WORDS[6]);
    s.log(EVERYTHING_WORDS[7]);
    s.newline();
    Ok(())
}

// ---- the race ----

/// Runs a race: sets `arcade_kind` 1 and the log's indent 0; for an arcade race's first stage
/// (the rally-stage flag 0, the mode flag 2) takes a new noise seed (`math::next_random` into
/// `noise_seed`); sets `editor_mode` 0 and `replay_mode` 1; then takes the settings in, sets up the
/// arcade's cars, sets the next race flag 1, the start states, loads the stage, opens the replay's
/// header and file, sets `after_credits` 0, runs the race (`drive::play_race`), closes the replay,
/// tidies everything and takes the settings out.
pub(super) fn run_race(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let race = o.race;
    race.physics.split_tables_mut().arcade_kind = 1;
    crate::logfile::write_line::state::INDENT.set(0);
    if race.shared.state.results.race_stage.get() == 0
        && race.shared.state.settings.run_kind.get() == 2
    {
        race.shared
            .tables()
            .noise_seed
            .set(race.shared.random.next_random());
    }
    race.shared.switches.borrow_mut().editor_mode = 0u32;
    race.course.number_vars.get().replay_mode.set(1u32);
    crate::race::settings_copy::copy_settings_in(race, o.gfx, &o.saves.best_tables)?;
    crate::race::race_cars::set_up_arcade_cars(race)?;
    race.shared.vars.get().settings_words[1].set(1u32);
    crate::race::race_shell::set_start_states(race, &o.net.race_slot)?;
    crate::race::stage_loading::load_stage(race, o)?;
    record_replay_header(race);
    crate::race::replay::open_replay(race, o)?;
    race.shared.view.get().after_credits.set(0u32);
    crate::drive::play_race::play_race(race, o)?;
    crate::race::replay::close_replay(race, o.system)?;
    tidy_everything(race, o)?;
    crate::race::settings_copy::copy_settings_out(race, &o.saves.best_tables)?;
    Ok(())
}

// ---- above the race ----

/// Runs `race::release_stage_textures` and `scene::reset_texture_tables`.
pub fn release_textures(scene: &crate::scene::owner::Scene, gfx: &crate::gfx::owner::Gfx) {
    crate::race::race_shell::release_stage_textures(gfx);
    crate::scene::reset_texture_tables::reset_texture_tables(scene);
}

/// Sets `mask_kind` 0 and runs `race::run_race`.
fn start_race(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    o.race.shared.switches.borrow_mut().mask_kind = 0u32;
    run_race(o)
}

/// Runs `race::render3d_tidy`, `race::objects_tidy`, `race::release_textures` and
/// `race::start_race`.
pub(super) fn race_from_menu(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    // A trouble the game can go back from (a stage's data that cannot be read) tears the race
    // down as when the player quits it, then goes on up to the main menu.
    match crate::race::dialogs::guarded(|| race_from_menu_plain(o)) {
        Ok(result) => result,
        Err(reason) => {
            tear_down_after_trouble(o);
            crate::race::dialogs::back(&reason)
        }
    }
}

/// Puts the game as it is at the main menu after a race that stopped half way: the replay
/// closed, everything tidied (the controls released, the stage and the race's sounds freed),
/// the settings taken out, then the menus' screen mode, sounds and scene. Each step is tried
/// whatever the one before did: a race that failed early has not made all of them.
fn tear_down_after_trouble(o: crate::owners::Owners<'static>) {
    use crate::race::single_rally::{Call, step};
    let race = o.race;
    let tried = |what: &str, r: Result<(), Refused>| {
        if let Err(why) = r {
            crate::program_log::log(&format!("recover: {what} could not be done: {why:?}"));
        }
    };
    tried(
        "closing the replay",
        crate::race::replay::close_replay(race, o.system),
    );
    tried(
        "tidying the race",
        tidy_everything_as(Tidy { forgiving: true }, race, o),
    );
    tried(
        "taking the settings out",
        crate::race::settings_copy::copy_settings_out(race, &o.saves.best_tables),
    );
    for call in [
        Call::StopMusic,
        Call::SetMenuScreenMode,
        Call::ClearRaceSoundObjects,
        Call::ApplySettings,
        Call::SetUpEngine,
        Call::SetUpFrontEndScene,
        Call::SetUpEngine,
        Call::TakeKey,
    ] {
        tried(&format!("{call:?}"), step(o, call));
    }
}

fn race_from_menu_plain(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    crate::race::stage_free::render3d_tidy(o.race, &o.scene.scratch)?;
    crate::race::stage_free::objects_tidy(o.race, &o.scene.scratch)?;
    release_textures(o.scene, o.gfx);
    start_race(o)
}
