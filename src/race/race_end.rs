//! The race's end and tidying (`sub_444AC7` "SHAUNTIDYEVERYTHING", `sub_444940`
//! "LOADLEVEL_TIDY", IDA 2026-10-17), and the replay header's record (`sub_460246`). Here
//! their steps.

use crate::errors::Refused;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::replay_work_store::REPLAY_WORK;
use crate::race::state_store::RACE_STATE;
use crate::race::switches_store::FRAME_SWITCHES;
use crate::race::tables_store::RACE_TABLES;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;

/// Where a header setting lives in the race's variables (its first place).
pub fn setting_cell(s: rc2k_core::race::replay::Setting) -> &'static crate::session::Dword {
    use rc2k_core::race::replay::Setting::*;
    match s {
        Seed => &RACE_TABLES.noise_seed,
        StartPlace => &RACE_TABLES.start_place,
        ArcadeCars => &RACE_TABLES.arcade_cars,
        Stage => &RACE_VARS.stage_number,
        Mode => &RACE_VARS.run_kind,
        Players => &RACE_VIEW.players,
        Cars => &RACE_VIEW.race_cars,
        Kind(n) => &CAR_BLOCKS.cars[n as usize].car_kind,
        GameOver(n) => &CAR_BLOCKS.cars[n as usize].stop_state,
    }
}

/// Where a header setting lives in the replay's car header (`dword_719B14` ..).
pub fn header_cell(s: rc2k_core::race::replay::Setting) -> &'static crate::session::Dword {
    use rc2k_core::race::replay::Setting::*;
    match s {
        Stage => &REPLAY_WORK.header_stage,
        Mode => &REPLAY_WORK.header_mode,
        Players => &REPLAY_WORK.header_players,
        Cars => &REPLAY_WORK.header_cars,
        Kind(0) => &REPLAY_WORK.header_kind_0,
        Kind(1) => &REPLAY_WORK.header_kind_1,
        Kind(2) => &REPLAY_WORK.header_kind_2,
        Kind(3) => &REPLAY_WORK.header_kind_3,
        StartPlace => &REPLAY_WORK.header_start_place,
        ArcadeCars => &REPLAY_WORK.header_arcade_cars,
        Seed => &REPLAY_WORK.header_seed,
        GameOver(0) => &REPLAY_WORK.header_game_over_0,
        GameOver(1) => &REPLAY_WORK.header_game_over_1,
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
/// `race::record_replay_header` (0x460246): the race's settings (the arcade's seed, start
/// place and count, the stage, the mode, the players, the cars, the first four kinds, the
/// two players' absences) into the car header (`dword_719B14` ..), each logged as
/// "[replayheader.x]" (`log_write_l1`) and its value (`log_int_l1_nl`). Every register
/// kept (the logs inside `pusha`); it reads none.
pub fn record_replay_header() {
    // Converted: `rc2k_core::race::replay::HEADER_SETTINGS`.
    use rc2k_core::race::replay::HEADER_SETTINGS;
    for s in HEADER_SETTINGS {
        header_cell(s).set(setting_cell(s).get());
    }
    for (n, &s) in HEADER_SETTINGS.iter().enumerate() {
        crate::logfile::log_write::log_nul(&REPLAY_WORDS[n], false, 1);
        crate::logfile::log_int::log_int(header_cell(s).get() as i32, true, 1);
    }
}

/// An object's variables reset for a race: the six timed slots' start and playing flags
/// and the other slots', the hit and cue sounds none (-2), the cue queue's place none,
/// the object not in use, the revs, throttle, speed and the wheels' first shares 0.
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
/// dword_51659C .. dword_5165E8: set 0.
fn clear_stage_flags() {
    let s = &crate::sound::owner::sound().race.stage;
    for flag in [
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
    ] {
        flag.set(0);
    }
}

/// A sound command whose only register the audio thread reads is eax (the others go
/// as 0).
pub fn command(number: u32, eax: u32) {
    let mut registers = [0; 7];
    registers[0] = eax;
    crate::sound::queue::send_command(number, registers);
}

/// `sound::wait_for_commands`: the queue run out (its ends of turn read no register).
pub fn wait_for_commands() {
    use crate::sound::wait_for_commands::{Game, wait_for_commands};
    wait_for_commands(&mut Game([0; 7]));
}

/// `race::reset_race_sounds` (0x40D378): the sound commands waited for, command 7 (with
/// its end of turn), waited again, sounds held back, the race clock started; the eight
/// race sound objects' 32 fields reset (`OBJECT_VALUES`); eleven variables 0; the 0x48
/// voices' +0 0x1888FFFF and +0x14 -2; `dword_5165EC`, `dword_518054` 0, the EAX
/// environment 1.0; command 2 with eax 3; `dword_5124E4` 0. Every register kept; the
/// commands' registers but those the audio thread reads (command 2's eax) go as 0.
pub fn reset_race_sounds(v: &SoundVars) -> Result<(), Refused> {
    wait_for_commands();
    command(7, 0);
    command(0xA, 7);
    wait_for_commands();
    crate::sound::hold_sounds_back::hold_sounds_back();
    crate::sound::race_clock::start_race_clock(crate::system::tick_count::tick_count());
    for o in &crate::sound::owner::sound().race.sounds.objects {
        reset_object(o);
    }
    clear_stage_flags();
    for voice in v.voices.iter() {
        voice[0].set(0x1888_FFFF);
        voice[5].set(0xFFFF_FFFE);
    }
    crate::sound::owner::sound().race.stage.shared_count.set(0);
    crate::sound::owner::sound()
        .race
        .spot_picks
        .spot_work_a
        .set(0);
    crate::audio::store::ENVIRONMENT.set(0x3F80_0000u32);
    command(2, 3);
    crate::sound::owner::sound().race.clock.started.set(0);
    Ok(())
}

/// Where `reset_race_sounds` finds the voices (the game's,
/// or a test's: the audio thread reads the game's).
pub struct SoundVars {
    pub voices: &'static [[crate::session::Dword; 20]; 72],
}

impl SoundVars {
    pub fn game() -> Self {
        Self {
            voices: &crate::sound::owner::sound().race.spot_voices.voices,
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
    /// `log_write_l1_nl` of a word.
    fn log(&self, word: &std::ffi::CStr) {
        crate::logfile::log_write::log_nul(&word, true, 1);
    }

    /// `log_write_l1_nl` of the empty text (0x4E00B7).
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

/// `race::tidy_stage` (0x444940, "LOADLEVEL_TIDY"): each step with its word and the log
/// indented round it: the mud, the point records, the marks' tidy (the log off), the
/// fonts, the map, the textures, the course's two memories, the objects (then `g_inRace`
/// 0, `dword_606294` `dword_6235F4`); "finis". What the steps leave in eax, ebx and the
/// flags is read by no caller (taint at its one site, 0x444B56 in `tidy_everything`: 0 reads).
pub fn tidy_stage() -> Result<(), Refused> {
    use crate::race::stage_free::{
        COURSE, COURSE_2, FONTS, MUD, OBJECT_POINTS, POINT_RECORDS, free,
    };
    let s = Tidy;
    s.step(STAGE_WORDS[0], || free(&MUD))?;
    s.step(STAGE_WORDS[1], || free(&POINT_RECORDS))?;
    s.log(STAGE_WORDS[2]);
    s.indent(3);
    let enabled = crate::logfile::write_line::state::ENABLED
        .atomic()
        .swap(0, std::sync::atomic::Ordering::Relaxed);
    crate::effects::tidy_marks::tidy_marks(crate::effects::owner::effects())?;
    crate::logfile::write_line::state::ENABLED.set(enabled);
    s.indent(-3);
    s.step(STAGE_WORDS[3], || free(&FONTS))?;
    s.step(STAGE_WORDS[4], crate::race::game_map::free_game_map)?;
    s.step(STAGE_WORDS[5], || {
        crate::race::race_shell::release_stage_textures();
        Ok(())
    })?;
    s.step(STAGE_WORDS[6], || free(&COURSE_2))?;
    s.step(STAGE_WORDS[7], || free(&COURSE))?;
    s.log(STAGE_WORDS[8]);
    s.indent(3);
    free(&OBJECT_POINTS)?;
    crate::race::owner::race().course.numbers.in_race.set(0);
    crate::race::owner::race()
        .frame
        .view_and_records
        .view_vars
        .view_scale
        .set_bits(RACE_VARS.kept_scale.get());
    s.indent(-3);
    s.log(STAGE_WORDS[9]);
    Ok(())
}

/// `race::tidy_everything` (0x444AC7, "SHAUNTIDYEVERYTHING"): a line break, a rule,
/// "START" (an empty indent), then each step with its word: the controls, the game mode,
/// `tidy_stage`, the sounds (then `dword_63DB74` and `dword_63DB78` 1); "finis", a rule, a
/// line break. Reads no register; of what it leaves its callers read only esi, which it
/// keeps (taint at all six sites: 0 reads of the rest).
pub fn tidy_everything() -> Result<(), Refused> {
    let s = Tidy;
    s.newline();
    s.log(EVERYTHING_WORDS[0]);
    s.log(EVERYTHING_WORDS[1]);
    s.indent(3);
    s.indent(-3);
    s.step(EVERYTHING_WORDS[2], || {
        crate::race::race_shell::release_controls();
        Ok(())
    })?;
    s.step(EVERYTHING_WORDS[3], || Ok(()))?;
    s.step(EVERYTHING_WORDS[4], tidy_stage)?;
    s.log(EVERYTHING_WORDS[5]);
    s.indent(3);
    reset_race_sounds(&SoundVars::game())?;
    RACE_VIEW.view_on[0].set(1u32);
    RACE_VIEW.view_on[1].set(1u32);
    s.indent(-3);
    s.log(EVERYTHING_WORDS[6]);
    s.log(EVERYTHING_WORDS[7]);
    s.newline();
    Ok(())
}

// ---- the race ----

/// `race::run_race` (0x443F0B): a race: `dword_60101C` 1, the log's indent 0; for an
/// arcade race's first stage (`dword_75AFF4` 0, `dword_75ACD8` 2) a new noise seed
/// (`math::next_random` into `dword_60519C`); `dword_621514` 0, `dword_5C5CE0` 1; the
/// settings in, the arcade's cars, `dword_6238AC` 1, the start states, the stage loaded,
/// the replay's header and file, `dword_63DB84` 0, the race run (`drive::play_race`), the
/// replay closed, everything tidied, the settings out. `indent` is `g_logIndent`'s
/// address. Its callers read only esi of what it leaves, which every step keeps.
pub fn run_race() -> Result<(), Refused> {
    crate::race::owner::race()
        .physics
        .split_tables
        .arcade_kind
        .set(1);
    crate::logfile::write_line::state::INDENT.set(0);
    if RACE_STATE.results.race_stage.get() == 0 && RACE_STATE.settings.run_kind.get() == 2 {
        RACE_TABLES
            .noise_seed
            .set(crate::math::next_random::next_random());
    }
    FRAME_SWITCHES.editor_mode.set(0u32);
    crate::race::owner::race()
        .course
        .numbers
        .replay_mode
        .set_bits(1u32);
    crate::race::settings_copy::copy_settings_in()?;
    crate::race::race_cars::set_up_arcade_cars()?;
    RACE_VARS.settings_words[1].set(1u32);
    crate::race::race_shell::set_start_states()?;
    crate::race::stage_loading::load_stage()?;
    record_replay_header();
    crate::race::replay::open_replay()?;
    RACE_VIEW.after_credits.set(0u32);
    crate::drive::play_race::play_race()?;
    crate::race::replay::close_replay()?;
    tidy_everything()?;
    crate::race::settings_copy::copy_settings_out()?;
    Ok(())
}

// ---- above the race ----

/// `race::release_textures` (0x443396): `race::release_stage_textures`,
/// `scene::reset_texture_tables`. Reads no register; what it leaves (ecx, edi, eax, the
/// flags) no caller reads (taint at its ten sites).
pub fn release_textures() {
    crate::race::race_shell::release_stage_textures();
    crate::scene::reset_texture_tables::reset_texture_tables();
}

/// `race::start_race` (0x4278B7): `dword_621870` 0, `race::run_race`.
pub fn start_race() -> Result<(), Refused> {
    FRAME_SWITCHES.mask_kind.set(0u32);
    run_race()
}

/// `race::race_from_menu` (0x485570): `race::render3d_tidy`, `race::objects_tidy`,
/// `nullsub_6`, `race::release_textures`, `race::start_race`.
pub fn race_from_menu() -> Result<(), Refused> {
    crate::race::stage_free::render3d_tidy()?;
    crate::race::stage_free::objects_tidy()?;
    release_textures();
    start_race()
}
