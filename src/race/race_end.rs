//! The race's end and tidying (`race::tidy_everything` "SHAUNTIDYEVERYTHING", `race::tidy_stage`
//! "LOADLEVEL_TIDY"), and the replay header's record (`race::record_replay_header`).

use crate::effects::owner::Effects;
use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::audio::commands::Command;

/// Where a header setting lives in the race's variables (its first place).
pub fn setting_cell(
    race: &Race,
    s: rc2k_core::race::replay::Setting,
) -> &dyn crate::session::DwordLike {
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
pub fn header_cell(
    race: &Race,
    s: rc2k_core::race::replay::Setting,
) -> &crate::session::PlainDword {
    let work = race.replay_work();
    use rc2k_core::race::replay::Setting::*;
    match s {
        Stage => &work.header_stage,
        Mode => &work.header_mode,
        Players => &work.header_players,
        Cars => &work.header_cars,
        Kind(0) => &work.header_kind_0,
        Kind(1) => &work.header_kind_1,
        Kind(2) => &work.header_kind_2,
        Kind(3) => &work.header_kind_3,
        StartPlace => &work.header_start_place,
        ArcadeCars => &work.header_arcade_cars,
        Seed => &work.header_seed,
        GameOver(0) => &work.header_game_over_0,
        GameOver(1) => &work.header_game_over_1,
        Kind(_) | GameOver(_) => unreachable!("the header holds four kinds and two flags"),
    }
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
        header_cell(race, s).set(setting_cell(race, s).get());
    }
    for (n, &s) in HEADER_SETTINGS.iter().enumerate() {
        crate::logfile::log_write::log_nul(&REPLAY_WORDS[n], false, 1);
        crate::logfile::log_int::log_int(header_cell(race, s).get() as i32, true, 1);
    }
}

/// An object's variables reset for a race: the six timed slots' start and playing flags and the
/// other slots', the hit and cue sounds none (-2), the cue queue's place none, the object not in
/// use, the revs, throttle, speed and the wheels' first shares 0.
fn reset_object(o: &crate::sound::store::SoundObject) {
    for slot in o.slots.iter().chain([&o.gear_slot, &o.slot_b, &o.slot_c]) {
        slot.start.set(0);
        slot.playing.set(0);
    }
    o.hit_sound.set(0xFFFF_FFFE);
    o.cue_sound.set(0xFFFF_FFFE);
    o.cue_queue.count.set(0);
    o.cue_queue.place.set(0xFFFF_FFFF);
    o.in_use.set(0);
    o.revs.set(0);
    o.throttle.set(0);
    o.speed.set(0.0);
    for voice in &o.voices {
        voice.share_a.set(0.0);
    }
}
/// The race sound stage's flags (the run of dwords in the stage block): all set to 0.
fn clear_stage_flags(sound: &crate::sound::owner::Sound) {
    use crate::session::DwordLike;
    let s = sound.race.stage.get();
    let flags: [&dyn DwordLike; 11] = [
        &s.once[0],
        &s.once[1],
        &s.once[2],
        &s.timed_arm,
        &s.timed_waiting,
        &s.timed_slot.start,
        &s.timed_slot.playing,
        &s.loop_volume_a,
        &s.loop_volume_b,
        &s.random[0],
        &s.random[1],
    ];
    for flag in flags {
        flag.set(0);
    }
}

/// Waits until the sound command queue has run out.
pub fn wait_for_commands(o: crate::owners::Owners<'static>) {
    crate::sound::wait_for_commands::wait_in_game(o);
}

/// Resets the race's sounds: waits for the sound commands, sends command 7 and waits again, holds
/// the sounds back and starts the race clock; resets the eight race sound objects' 32 fields
/// and eleven variables to 0; sets the 0x48 voices' first dword to 0x1888FFFF and
/// their dword at +0x14 to -2; clears two more sound flags, sets the EAX environment to 1.0, sends
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
    crate::sound::race_clock::start_race_clock(crate::system::tick_count::tick_count(), sound);
    for o in &sound.race.sounds.get().objects {
        reset_object(o);
    }
    clear_stage_flags(sound);
    for voice in v.voices.iter() {
        voice.id.set(crate::sound::spot_voices::SpotVoice::FREE_ID);
        voice.sound.set(0xFFFF_FFFE);
    }
    sound.race.stage.get().shared_count.set(0);
    audio.environment.set(0x3F80_0000u32);
    crate::sound::queue::send_command_in(audio, Command::Room(3));
    sound.race.clock.get().started.set(0);
    Ok(())
}

/// Where `reset_race_sounds` finds the voices (the game's, or a test's).
pub struct SoundVars<'a> {
    pub voices: &'a [crate::sound::spot_voices::SpotVoice; 72],
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

struct Tidy;

impl Tidy {
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
        call()?;
        self.indent(-3);
        Ok(())
    }
}

/// Tidies the stage ("LOADLEVEL_TIDY"), each step with its word and the log indented round it: the
/// mud, the point records, the marks' tidy (the log off), the fonts, the map, the textures, the
/// course's two memories, the objects (then `in_race` 0, `view_scale` `kept_scale`); "finis".
pub fn tidy_stage(
    race: &Race,
    effects: &Effects,
    gfx: &crate::gfx::owner::Gfx,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    use crate::race::stage_free::{
        COURSE, COURSE_2, MUD, OBJECT_POINTS, POINT_RECORDS, free, free_game_fonts,
    };
    let s = Tidy;
    s.step(STAGE_WORDS[0], || free(race, &MUD))?;
    s.step(STAGE_WORDS[1], || free(race, &POINT_RECORDS))?;
    s.log(STAGE_WORDS[2]);
    s.indent(3);
    let enabled = crate::logfile::write_line::state::ENABLED
        .atomic()
        .swap(0, std::sync::atomic::Ordering::Relaxed);
    crate::effects::tidy_marks::tidy_marks(effects)?;
    crate::logfile::write_line::state::ENABLED.set(enabled);
    s.indent(-3);
    s.step(STAGE_WORDS[3], || free_game_fonts(gfx))?;
    s.step(STAGE_WORDS[4], || {
        crate::race::game_map::free_game_map(race)
    })?;
    s.step(STAGE_WORDS[5], || {
        crate::race::race_shell::release_stage_textures(gfx);
        Ok(())
    })?;
    s.step(STAGE_WORDS[6], || free(race, &COURSE_2))?;
    s.step(STAGE_WORDS[7], || free(race, &COURSE))?;
    s.log(STAGE_WORDS[8]);
    s.indent(3);
    free(race, &OBJECT_POINTS)?;
    race.course.numbers.get().in_race.set(false);
    crate::workers::set_game_part_racing(false);
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
    let (input, sound) = (o.input, o.sound);
    let shared = &race.shared;
    let s = Tidy;
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
        tidy_stage(race, o.race.effects.get(), o.gfx, o.audio)
    })?;
    #[cfg(feature = "remote")]
    crate::race::course_hash::log_hashes(race, o.scene, "free");
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
pub fn run_race(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let race = o.race;
    race.physics.split_tables().arcade_kind.set(1);
    crate::logfile::write_line::state::INDENT.set(0);
    if race.shared.state.results.race_stage.get() == 0
        && race.shared.state.settings.run_kind.get() == 2
    {
        race.shared
            .tables()
            .noise_seed
            .set(crate::math::next_random::next_random());
    }
    race.shared.switches.get().editor_mode.set(0u32);
    race.course.numbers.get().replay_mode.set(1u32);
    crate::race::settings_copy::copy_settings_in(race, o.gfx)?;
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
    crate::race::settings_copy::copy_settings_out(race)?;
    Ok(())
}

// ---- above the race ----

/// Runs `race::release_stage_textures` and `scene::reset_texture_tables`.
pub fn release_textures(scene: &crate::scene::owner::Scene, gfx: &crate::gfx::owner::Gfx) {
    crate::race::race_shell::release_stage_textures(gfx);
    crate::scene::reset_texture_tables::reset_texture_tables(scene);
}

/// Sets `mask_kind` 0 and runs `race::run_race`.
pub fn start_race(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    o.race.shared.switches.get().mask_kind.set(0u32);
    run_race(o)
}

/// Runs `race::render3d_tidy`, `race::objects_tidy`, `race::release_textures` and
/// `race::start_race`.
pub fn race_from_menu(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    crate::race::stage_free::render3d_tidy(o.race)?;
    crate::race::stage_free::objects_tidy(o.race)?;
    release_textures(o.scene, o.gfx);
    start_race(o)
}
