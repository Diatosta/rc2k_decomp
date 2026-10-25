//! The race's end and tidying (`sub_444AC7` "SHAUNTIDYEVERYTHING", `sub_444940`
//! "LOADLEVEL_TIDY", IDA 2026-10-17), and the replay header's record (`sub_460246`). Here
//! their steps.

use crate::errors::Refused;
use crate::framework::rva;
use crate::store::{game_addr, game_read, game_write};

/// The replay's car header (`dword_719B14` ..).
pub const HEADER: usize = 0x719B14;

/// Where a header setting lives in the race's variables (its first place).
pub fn setting_place(s: rc2k_core::race::replay::Setting) -> usize {
    use rc2k_core::race::replay::Setting::*;
    match s {
        Seed => 0x60519C,
        StartPlace => 0x6051BC,
        ArcadeCars => 0x6051B4,
        Stage => 0x623850,
        Mode => 0x62389C,
        Players => 0x63DB94,
        Cars => 0x63DB90,
        Kind(n) => 0x70F410 + 0xFE0 * n as usize,
        GameOver(n) => 0x70F49C + 0xFE0 * n as usize,
    }
}

/// "[replayheader.arcrandom]" .. "[replayheader.gameoverflag2]", logged in the order of
/// `HEADER_SETTINGS`.
const REPLAY_WORDS: [usize; 13] = [
    0x4DDCBC, 0x4DDCD5, 0x4DDCF0, 0x4DDD0A, 0x4DDD1F, 0x4DDD34, 0x4DDD49, 0x4DDD60, 0x4DDD74,
    0x4DDD88, 0x4DDD9C, 0x4DDDB0, 0x4DDDCD,
];
/// `race::record_replay_header` (0x460246): the race's settings (the arcade's seed, start
/// place and count, the stage, the mode, the players, the cars, the first four kinds, the
/// two players' absences) into the car header (`dword_719B14` ..), each logged as
/// "[replayheader.x]" (`log_write_l1`) and its value (`log_int_l1_nl`). Every register
/// kept (the logs inside `pusha`); it reads none.
pub unsafe fn record_replay_header() {
    // Converted: `rc2k_core::race::replay::{HEADER_SETTINGS, Setting::offset}`.
    use rc2k_core::race::replay::HEADER_SETTINGS;
    let to = |s: rc2k_core::race::replay::Setting| rva(HEADER) + s.offset() as usize;
    unsafe {
        for s in HEADER_SETTINGS {
            game_write(to(s), game_read::<u32>(rva(setting_place(s))));
        }
        for (n, &s) in HEADER_SETTINGS.iter().enumerate() {
            crate::logfile::log_write::log_write(
                game_addr(rva(REPLAY_WORDS[n])) as *const u8,
                false,
                1,
            );
            crate::logfile::log_int::log_int(game_read::<u32>(to(s)) as i32, true, 1);
        }
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
    let s = &crate::sound::store::STAGE_SOUNDS;
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
const VOICES_RVA: usize = rva(0x5168C0); // unk_5168C0: 0x48 records of 0x50

/// A sound command whose only register the audio thread reads is eax (the others go
/// as 0).
pub fn command(number: u32, eax: u32) {
    let mut registers = [0; 7];
    registers[0] = eax;
    unsafe {
        crate::sound::queue::send_command(&crate::sound::queue::Queue::game(), number, registers)
    };
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
/// Refuses, before anything, a sound object missing (the original writes through 0).
pub fn reset_race_sounds(v: &SoundVars) -> Result<(), Refused> {
    unsafe {
        let objects: Vec<u32> = (0..8)
            .map(|n| ((v.objects + 4 * n) as *const u32).read_unaligned())
            .collect();
        if objects.contains(&0) {
            return Err(crate::errors::BadInput("a race sound object missing").into());
        }
        wait_for_commands();
        command(7, 0);
        command(0xA, 7);
        wait_for_commands();
        crate::sound::hold_sounds_back::hold_sounds_back();
        crate::sound::race_clock::start_race_clock(crate::system::tick_count::tick_count());
        for &o in &objects {
            reset_object(crate::sound::store::SoundObject::at(o));
        }
        clear_stage_flags();
        let first = v.voices;
        for k in 0..0x48u32 {
            let at = first + 0x50 * k;
            (at as *mut u32).write_unaligned(0x1888_FFFF);
            ((at + 0x14) as *mut u32).write_unaligned(0xFFFF_FFFE);
        }
        crate::sound::store::STAGE_SOUNDS.shared_count.set(0);
        crate::sound::store::SOUND_EFFECTS.spot_work_a.set(0);
        crate::audio::store::ENVIRONMENT.set(0x3F80_0000u32);
        command(2, 3);
        crate::sound::store::RACE_CLOCK.started.set(0);
    }
    Ok(())
}

/// Where `reset_race_sounds` finds the sound objects' table and the voices (the game's,
/// or a test's: the audio thread reads the game's).
pub struct SoundVars {
    pub objects: u32,
    pub voices: u32,
}

impl SoundVars {
    pub fn game() -> Self {
        Self {
            objects: crate::sound::store::RACE_SOUNDS.table.as_ptr() as usize as u32,
            voices: game_addr(VOICES_RVA) as u32,
        }
    }
}

// ---- the tidying ----

use crate::race::loadlevel::LoadVars;

/// "LOADLEVEL_TIDY : tidymud" .. "finis".
const STAGE_WORDS: [usize; 10] = [
    0x4DC1D7, 0x4DC1F0, 0x4DC20F, 0x4DC22D, 0x4DC250, 0x4DC271, 0x4DC291, 0x4DC2B3, 0x4DC2D3,
    0x4DC2F1,
];
/// "-----", "SHAUNTIDYEVERYTHING : START" .. "finis", "-----".
const EVERYTHING_WORDS: [usize; 8] = [
    0x4DC308, 0x4DC32A, 0x4DC346, 0x4DC36D, 0x4DC396, 0x4DC3BB, 0x4DC3DD, 0x4DC3F9,
];

struct Tidy<'a> {
    v: &'a LoadVars,
}

impl Tidy<'_> {
    /// `log_write_l1_nl` of a word.
    fn log(&self, word: usize) {
        unsafe { crate::logfile::log_write::log_write(game_addr(rva(word)) as *const u8, true, 1) }
    }

    /// `log_write_l1_nl` of the empty text (0x4E00B7).
    fn newline(&self) {
        self.log(0x4E00B7);
    }

    fn indent(&self, by: i32) {
        unsafe {
            let at = self.v.indent as *mut u32;
            at.write_unaligned(at.read_unaligned().wrapping_add(by as u32));
        }
    }

    /// The word, the indent in, the call, the indent out.
    fn step(&self, word: usize, call: impl FnOnce() -> Result<(), Refused>) -> Result<(), Refused> {
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
pub unsafe fn tidy_stage(v: &LoadVars) -> Result<(), Refused> {
    use crate::race::stage_free::{
        COURSE, COURSE_2, FONTS, MUD, OBJECT_POINTS, POINT_RECORDS, free,
    };
    let s = Tidy { v };
    unsafe {
        s.step(STAGE_WORDS[0], || free(&MUD))?;
        s.step(STAGE_WORDS[1], || free(&POINT_RECORDS))?;
        s.log(STAGE_WORDS[2]);
        s.indent(3);
        let enabled = (v.log_enabled as *const u32).read_unaligned();
        (v.log_enabled as *mut u32).write_unaligned(0);
        crate::marks::tidy_marks::tidy_marks()?;
        (v.log_enabled as *mut u32).write_unaligned(enabled);
        s.indent(-3);
        s.step(STAGE_WORDS[3], || free(&FONTS))?;
        s.step(STAGE_WORDS[4], || crate::race::game_map::free_game_map())?;
        s.step(STAGE_WORDS[5], || {
            crate::race::race_shell::release_stage_textures();
            Ok(())
        })?;
        s.step(STAGE_WORDS[6], || free(&COURSE_2))?;
        s.step(STAGE_WORDS[7], || free(&COURSE))?;
        s.log(STAGE_WORDS[8]);
        s.indent(3);
        free(&OBJECT_POINTS)?;
        (v.in_race as *mut u32).write_unaligned(0);
        game_write(rva(0x606294), game_read::<u32>(rva(0x6235F4)));
        s.indent(-3);
        s.log(STAGE_WORDS[9]);
    }
    Ok(())
}

/// `race::tidy_everything` (0x444AC7, "SHAUNTIDYEVERYTHING"): a line break, a rule,
/// "START" (an empty indent), then each step with its word: the controls, the game mode,
/// `tidy_stage`, the sounds (then `dword_63DB74` and `dword_63DB78` 1); "finis", a rule, a
/// line break. Reads no register; of what it leaves its callers read only esi, which it
/// keeps (taint at all six sites: 0 reads of the rest).
pub unsafe fn tidy_everything(v: &LoadVars) -> Result<(), Refused> {
    let s = Tidy { v };
    unsafe {
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
        s.step(EVERYTHING_WORDS[4], || tidy_stage(v))?;
        s.log(EVERYTHING_WORDS[5]);
        s.indent(3);
        reset_race_sounds(&SoundVars::game())?;
        game_write(rva(0x63DB74), 1u32);
        game_write(rva(0x63DB78), 1u32);
        s.indent(-3);
        s.log(EVERYTHING_WORDS[6]);
        s.log(EVERYTHING_WORDS[7]);
        s.newline();
    }
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
pub unsafe fn run_race(indent: u32) -> Result<(), Refused> {
    unsafe {
        game_write(rva(0x60101C), 1u32);
        (indent as *mut u32).write_unaligned(0);
        if game_read::<u32>(rva(0x75AFF4)) == 0 && game_read::<u32>(rva(0x75ACD8)) == 2 {
            game_write(rva(0x60519C), crate::math::next_random::next_random());
        }
        game_write(rva(0x621514), 0u32);
        game_write(rva(0x5C5CE0), 1u32);
        crate::race::settings_copy::copy_settings_in()?;
        crate::race::race_cars::set_up_arcade_cars()?;
        game_write(rva(0x6238AC), 1u32);
        crate::race::race_shell::set_start_states()?;
        crate::race::stage_loading::load_stage()?;
        record_replay_header();
        crate::race::replay::open_replay()?;
        game_write(rva(0x63DB84), 0u32);
        crate::drive::play_race::play_race()?;
        crate::race::replay::close_replay()?;
        tidy_everything(&LoadVars::game())?;
        crate::race::settings_copy::copy_settings_out()?;
    }
    Ok(())
}

// ---- above the race ----

/// `race::release_textures` (0x443396): `race::release_stage_textures`,
/// `scene::reset_texture_tables`. Reads no register; what it leaves (ecx, edi, eax, the
/// flags) no caller reads (taint at its ten sites).
pub unsafe fn release_textures() {
    crate::race::race_shell::release_stage_textures();
    unsafe { crate::scene::reset_texture_tables::reset_texture_tables() };
}

/// `race::start_race` (0x4278B7): `dword_621870` 0, `race::run_race`.
pub unsafe fn start_race() -> Result<(), Refused> {
    unsafe {
        game_write(rva(0x621870), 0u32);
        run_race(crate::logfile::write_line::indent_place()) // g_logIndent
    }
}

/// `race::race_from_menu` (0x485570): `race::render3d_tidy`, `race::objects_tidy`,
/// `nullsub_6`, `race::release_textures`, `race::start_race`.
pub unsafe fn race_from_menu() -> Result<(), Refused> {
    unsafe {
        crate::race::stage_free::render3d_tidy()?;
        crate::race::stage_free::objects_tidy()?;
        release_textures();
        start_race()
    }
}
