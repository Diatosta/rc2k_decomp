//! The race's end and tidying (`sub_444AC7` "SHAUNTIDYEVERYTHING", `sub_444940`
//! "LOADLEVEL_TIDY", IDA 2026-10-17), and the replay header's record (`sub_460246`). Here
//! their steps.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::service::harness::{Region, at};

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

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
/// The log calls' places: `log_write_l1`, `log_int_l1_nl`.
pub const LOG_WRITE: u32 = 1;
pub const LOG_INT_NL: u32 = 2;

/// `race::record_replay_header` (0x460246): the race's settings (the arcade's seed, start
/// place and count, the stage, the mode, the players, the cars, the first four kinds, the
/// two players' absences) into the car header (`dword_719B14` ..), each logged as
/// "[replayheader.x]" and its value. Every register kept (the logs inside `pusha`).
pub unsafe fn record_replay_header(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::replay::{HEADER_SETTINGS, Setting::offset}`.
    use rc2k_core::race::replay::HEADER_SETTINGS;
    let g = *entry;
    let to = |s: rc2k_core::race::replay::Setting| rva(HEADER) + s.offset() as usize;
    unsafe {
        for s in HEADER_SETTINGS {
            game_write(to(s), game_read::<u32>(rva(setting_place(s))));
        }
        for (n, &s) in HEADER_SETTINGS.iter().enumerate() {
            let mut r = Regs {
                eax: game_addr(rva(REPLAY_WORDS[n])) as u32,
                ..g
            };
            part(LOG_WRITE, &mut r)?;
            let mut r = Regs {
                eax: game_read(to(s)),
                ..g
            };
            part(LOG_INT_NL, &mut r)?;
        }
    }
    Ok(g)
}

pub fn replay_header_regions() -> Vec<Region> {
    rc2k_core::race::replay::HEADER_SETTINGS
        .iter()
        .map(|s| at(rva(HEADER) + s.offset() as usize))
        .collect()
}

/// The sound parts of `reset_race_sounds`, by place.
pub const WAIT: u32 = 1; // sound::wait_for_commands
pub const COMMAND_7: u32 = 2; // sound::send_command_7
pub const HOLD_BACK: u32 = 3; // sound::hold_sounds_back
pub const CLOCK: u32 = 4; // sound::start_race_clock
pub const COMMAND_2: u32 = 5; // sound::send_command_2

const SOUND_OBJECTS_RVA: usize = rva(0x512A10); // g_raceSoundObjects: 8 pointers
/// Each object's dwords set: (offset, value).
const OBJECT_VALUES: [(u32, u32); 32] = [
    (0x150, 0),
    (0x154, 0),
    (0x168, 0),
    (0x16C, 0),
    (0x180, 0),
    (0x184, 0),
    (0x198, 0),
    (0x19C, 0),
    (0x264, 0),
    (0x268, 0),
    (0x1B0, 0),
    (0x1B4, 0),
    (0x224, 0),
    (0x228, 0),
    (0x1C8, 0),
    (0x1CC, 0),
    (0x1E0, 0),
    (0x1E4, 0),
    (0x1FC, 0),
    (0x200, 0),
    (0x238, 0xFFFF_FFFE),
    (0x24C, 0xFFFF_FFFE),
    (0x2A4, 0),
    (0x29C, 0xFFFF_FFFF),
    (0x8, 0),
    (0x14, 0),
    (0x18, 0),
    (0x1C, 0),
    (0x44, 0),
    (0x58, 0),
    (0x6C, 0),
    (0x80, 0),
];
/// dword_51659C .. dword_5165E8: set 0.
const CLEARED: [usize; 11] = [
    0x51659C, 0x5165A0, 0x5165A4, 0x5165A8, 0x5165C8, 0x5165D0, 0x5165D4, 0x5165BC, 0x5165C0,
    0x5165E4, 0x5165E8,
];
const VOICES_RVA: usize = rva(0x5168C0); // unk_5168C0: 0x48 records of 0x50
const ENVIRONMENT_RVA: usize = rva(0x6D1DD4); // g_eaxEnvironment

/// `race::reset_race_sounds` (0x40D378): the sound commands waited for, command 7 (with
/// its end of turn), waited again, sounds held back, the race clock started; the eight
/// race sound objects' 32 fields reset (`OBJECT_VALUES`); eleven variables 0; the 0x48
/// voices' +0 0x1888FFFF and +0x14 -2; `dword_5165EC`, `dword_518054` 0, the EAX
/// environment 1.0; command 2 with eax 3; `dword_5124E4` 0. Every register kept.
/// Refuses, before anything, a sound object missing (the original writes through 0).
pub unsafe fn reset_race_sounds(entry: &Regs, v: &SoundVars, part: Part) -> Result<Regs, Refused> {
    let g = *entry;
    unsafe {
        let objects: Vec<u32> = (0..8)
            .map(|n| ((v.objects + 4 * n) as *const u32).read_unaligned())
            .collect();
        if objects.contains(&0) {
            return Err(crate::errors::BadInput("a race sound object missing").into());
        }
        let mut r = g;
        part(WAIT, &mut r)?;
        part(COMMAND_7, &mut r)?;
        part(WAIT, &mut r)?;
        part(HOLD_BACK, &mut r)?;
        part(CLOCK, &mut r)?;
        for &o in &objects {
            for (off, v) in OBJECT_VALUES {
                ((o + off) as *mut u32).write_unaligned(v);
            }
        }
        for a in CLEARED {
            game_write(rva(a), 0u32);
        }
        let first = v.voices;
        for k in 0..0x48u32 {
            let at = first + 0x50 * k;
            (at as *mut u32).write_unaligned(0x1888_FFFF);
            ((at + 0x14) as *mut u32).write_unaligned(0xFFFF_FFFE);
        }
        game_write(rva(0x5165EC), 0u32);
        game_write(rva(0x518054), 0u32);
        game_write(ENVIRONMENT_RVA, 0x3F80_0000u32);
        r.eax = 3;
        r.ebx = first + 0x50 * 0x48;
        r.ecx = 0;
        part(COMMAND_2, &mut r)?;
        game_write(rva(0x5124E4), 0u32);
    }
    Ok(g)
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
            objects: game_addr(SOUND_OBJECTS_RVA) as u32,
            voices: game_addr(VOICES_RVA) as u32,
        }
    }
}

/// The game's sound object table and voices, as IDA shows them (for a test's copy).
pub const SOUND_OBJECTS_IDA: usize = 0x512A10;
pub const VOICES_IDA: usize = 0x5168C0;
pub const VOICES_LEN: usize = 0x48 * 0x50;
pub const OBJECT_LEN: usize = 0x2A8;

/// The plain variables `reset_race_sounds` writes.
pub fn sound_regions() -> Vec<Region> {
    let mut r: Vec<Region> = CLEARED.iter().map(|&a| at(rva(a))).collect();
    r.extend([
        at(rva(0x5165EC)),
        at(rva(0x518054)),
        at(ENVIRONMENT_RVA),
        at(rva(0x5124E4)),
    ]);
    r
}

/// `reset_race_sounds`' sound parts in the game: the Rust versions.
pub fn sound_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::sound::queue::{Queue, send_command, sent};
    use crate::sound::wait_for_commands::{Game, wait_for_commands};
    unsafe {
        match place {
            WAIT => wait_for_commands(&mut Game(sent(r))),
            COMMAND_7 => {
                // `pusha`, command 7 with the registers, `pop eax` (eax 7), the end of
                // turn (0xA) with them, `sub_409BA1` (a `cmp` only), `popa`.
                send_command(&Queue::game(), 7, sent(r));
                let with = Regs { eax: 7, ..*r };
                send_command(&Queue::game(), 0xA, sent(&with));
            }
            HOLD_BACK => crate::sound::hold_sounds_back::hold_sounds_back(),
            CLOCK => {
                crate::sound::race_clock::start_race_clock(crate::system::tick_count::tick_count())
            }
            _ => send_command(&Queue::game(), 2, sent(r)),
        }
    }
    Ok(())
}

// ---- the tidying ----

use crate::race::loadlevel::{LoadVars, or_in_game};

/// The tidying's calls by place, as IDA shows them.
pub const TIDY_CALLEES: [usize; 16] = [
    0,        // 0 (not used)
    0x449ECA, // 1 race::free_mud
    0x4433A7, // 2 race::render3d_tidy
    0x4D146D, // 3 marks::tidy_marks (the main session's)
    0x41B2BC, // 4 race::free_game_fonts
    0x41B907, // 5 race::free_game_map
    0x44E79F, // 6 race::release_stage_textures
    0x41F94D, // 7 race::free_course_list_2
    0x41F924, // 8 race::free_course_memory
    0x44346B, // 9 race::objects_tidy
    0x40142B, // 10 logfile::log_write_l1_nl
    0x4013DE, // 11 logfile::log_newline_l1
    0x409425, // 12 race::release_controls
    0x436B69, // 13 race::tidy_game_mode
    0x444940, // 14 race::tidy_stage
    0x45D28E, // 15 race::stop_race_sounds
];
const TIDY_LOG: u32 = 10;
const TIDY_NEWLINE: u32 = 11;
/// "LOADLEVEL_TIDY : tidymud" .. "finis".
const STAGE_WORDS: [usize; 10] = [
    0x4DC1D7, 0x4DC1F0, 0x4DC20F, 0x4DC22D, 0x4DC250, 0x4DC271, 0x4DC291, 0x4DC2B3, 0x4DC2D3,
    0x4DC2F1,
];
/// "-----", "SHAUNTIDYEVERYTHING : START" .. "finis", "-----".
const EVERYTHING_WORDS: [usize; 8] = [
    0x4DC308, 0x4DC32A, 0x4DC346, 0x4DC36D, 0x4DC396, 0x4DC3BB, 0x4DC3DD, 0x4DC3F9,
];

struct Tidy<'a, 'b> {
    g: Regs,
    v: &'b LoadVars,
    part: Part<'a>,
}

impl Tidy<'_, '_> {
    fn log(&mut self, word: usize) -> Result<(), Refused> {
        let mut r = Regs {
            eax: game_addr(rva(word)) as u32,
            ..self.g
        };
        (self.part)(TIDY_LOG, &mut r)
    }

    fn newline(&mut self) -> Result<(), Refused> {
        let mut r = self.g;
        (self.part)(TIDY_NEWLINE, &mut r)
    }

    fn indent(&mut self, by: i32) {
        unsafe {
            let at = self.v.indent as *mut u32;
            at.write_unaligned(at.read_unaligned().wrapping_add(by as u32));
        }
    }

    /// The word, the indent in, the call, the indent out.
    fn step(&mut self, word: usize, callee: u32) -> Result<(), Refused> {
        self.log(word)?;
        self.indent(3);
        (self.part)(callee, &mut self.g)?;
        self.indent(-3);
        Ok(())
    }
}

/// `race::tidy_stage` (0x444940, "LOADLEVEL_TIDY"): each step with its word and the log
/// indented round it: the mud, the point records, the marks' tidy (the log off), the
/// fonts, the map, the textures, the course's two memories, the objects (then `g_inRace`
/// 0, `dword_606294` `dword_6235F4`); "finis". Leaves the registers as the steps.
pub unsafe fn tidy_stage(entry: &Regs, v: &LoadVars, part: Part) -> Result<Regs, Refused> {
    let mut s = Tidy { g: *entry, v, part };
    unsafe {
        s.step(STAGE_WORDS[0], 1)?;
        s.step(STAGE_WORDS[1], 2)?;
        s.log(STAGE_WORDS[2])?;
        s.indent(3);
        let enabled = (v.log_enabled as *const u32).read_unaligned();
        (v.log_enabled as *mut u32).write_unaligned(0);
        (s.part)(3, &mut s.g)?;
        (v.log_enabled as *mut u32).write_unaligned(enabled);
        s.indent(-3);
        s.step(STAGE_WORDS[3], 4)?;
        s.step(STAGE_WORDS[4], 5)?;
        for (n, callee) in [(5, 6), (6, 7), (7, 8)] {
            s.step(STAGE_WORDS[n], callee)?;
        }
        s.log(STAGE_WORDS[8])?;
        s.indent(3);
        (s.part)(9, &mut s.g)?;
        (v.in_race as *mut u32).write_unaligned(0);
        game_write(rva(0x606294), game_read::<u32>(rva(0x6235F4)));
        s.indent(-3);
        s.log(STAGE_WORDS[9])?;
    }
    Ok(s.g)
}

/// `race::tidy_everything` (0x444AC7, "SHAUNTIDYEVERYTHING"): a line break, a rule,
/// "START" (an empty indent), then each step with its word: the controls, the game mode,
/// `tidy_stage`, the sounds (then `dword_63DB74` and `dword_63DB78` 1); "finis", a rule, a
/// line break. Leaves the registers as the steps.
pub unsafe fn tidy_everything(entry: &Regs, v: &LoadVars, part: Part) -> Result<Regs, Refused> {
    let mut s = Tidy { g: *entry, v, part };
    unsafe {
        s.newline()?;
        s.log(EVERYTHING_WORDS[0])?;
        s.log(EVERYTHING_WORDS[1])?;
        s.indent(3);
        s.indent(-3);
        s.step(EVERYTHING_WORDS[2], 12)?;
        s.step(EVERYTHING_WORDS[3], 13)?;
        s.step(EVERYTHING_WORDS[4], 14)?;
        s.log(EVERYTHING_WORDS[5])?;
        s.indent(3);
        (s.part)(15, &mut s.g)?;
        game_write(rva(0x63DB74), 1u32);
        game_write(rva(0x63DB78), 1u32);
        s.indent(-3);
        s.log(EVERYTHING_WORDS[6])?;
        s.log(EVERYTHING_WORDS[7])?;
        s.newline()?;
    }
    Ok(s.g)
}

/// The tidying's calls in the game: the Rust versions (each through `or_in_game`).
pub fn tidy_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        1..=9 | 12..=15 => or_in_game(TIDY_CALLEES[place as usize], r, |r| tidy_step(place, r)),
        _ => tidy_step(place, r),
    }
}

fn tidy_step(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::stage_free::{
        COURSE, COURSE_2, FONTS, MUD, OBJECT_POINTS, POINT_RECORDS, free,
    };
    unsafe {
        let freed = |r: &mut Regs, f| -> Result<(), Refused> {
            let g = free(r, f)?;
            r.eax = g.eax;
            r.ebx = g.ebx;
            Ok(())
        };
        match place {
            1 => freed(r, &MUD)?,
            2 => freed(r, &POINT_RECORDS)?,
            3 => *r = crate::marks::tidy_marks::tidy_marks(r)?,
            4 => freed(r, &FONTS)?,
            5 => crate::race::game_map::free_game_map()?,
            6 => {
                *r = crate::race::race_shell::release_stage_textures(
                    r,
                    &mut crate::race::race_shell::game_release_textures,
                )
            }
            7 => freed(r, &COURSE_2)?,
            8 => freed(r, &COURSE)?,
            9 => freed(r, &OBJECT_POINTS)?,
            TIDY_LOG => crate::logfile::log_write::log_write(r.eax as *const u8, true, 1),
            TIDY_NEWLINE => {
                crate::logfile::log_write::log_write(game_addr(rva(0x4E00B7)) as *const u8, true, 1)
            }
            12 => crate::race::race_shell::release_controls(),
            13 => {}
            14 => *r = tidy_stage(r, &LoadVars::game(), &mut tidy_part)?,
            _ => *r = reset_race_sounds(r, &SoundVars::game(), &mut sound_part)?,
        }
    }
    Ok(())
}

/// The plain variables the tidying writes.
pub fn tidy_regions() -> Vec<Region> {
    [0x606294usize, 0x6235F4, 0x63DB74, 0x63DB78]
        .iter()
        .map(|&a| at(rva(a)))
        .collect()
}

// ---- the race ----

/// `race::record_replay_header`'s log calls in the game.
pub fn replay_log_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        if place == LOG_WRITE {
            crate::logfile::log_write::log_write(r.eax as *const u8, false, 1);
        } else {
            crate::logfile::log_int::log_int(r.eax as i32, true, 1);
        }
    }
    Ok(())
}

/// `race::run_race`'s calls by place, as IDA shows them.
pub const RACE_CALLEES: [usize; 12] = [
    0,        // 0 (not used)
    0x4018B4, // 1 math::next_random
    0x446EBE, // 2 race::copy_settings_in
    0x4468FB, // 3 race::set_up_arcade_cars
    0x446D2E, // 4 race::set_start_states
    0x444040, // 5 race::load_stage
    0x460246, // 6 race::record_replay_header
    0x41E6EC, // 7 race::open_replay
    0x444CEE, // 8 sub_444CEE (the race's run: stream B's, in the game)
    0x41E85A, // 9 race::close_replay
    0x444AC7, // 10 race::tidy_everything
    0x4486FB, // 11 race::copy_settings_out
];

/// `race::run_race` (0x443F0B): a race: `dword_60101C` 1, the log's indent 0; for an
/// arcade race's first stage (`dword_75AFF4` 0, `dword_75ACD8` 2) a new noise seed
/// (`math::next_random` into `dword_60519C`); `dword_621514` 0, `dword_5C5CE0` 1; the
/// settings in, the arcade's cars, `dword_6238AC` 1, the start states, the stage loaded,
/// the replay's header and file, `dword_63DB84` 0, the race run (`sub_444CEE`), the
/// replay closed, everything tidied, the settings out. Leaves the registers as the last
/// step. `indent` is `g_logIndent`'s address.
pub unsafe fn run_race(entry: &Regs, indent: u32, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe {
        game_write(rva(0x60101C), 1u32);
        (indent as *mut u32).write_unaligned(0);
        if game_read::<u32>(rva(0x75AFF4)) == 0 && game_read::<u32>(rva(0x75ACD8)) == 2 {
            part(1, &mut g)?;
            game_write(rva(0x60519C), g.eax);
        }
        game_write(rva(0x621514), 0u32);
        game_write(rva(0x5C5CE0), 1u32);
        part(2, &mut g)?;
        part(3, &mut g)?;
        game_write(rva(0x6238AC), 1u32);
        for place in 4..=7 {
            part(place, &mut g)?;
        }
        game_write(rva(0x63DB84), 0u32);
        for place in 8..=11 {
            part(place, &mut g)?;
        }
    }
    Ok(g)
}

/// The race's calls in the game: the Rust versions (each through `or_in_game`), the
/// race's run the game's.
pub fn race_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        1 => {
            r.eax = unsafe { crate::math::next_random::next_random() };
            Ok(())
        }
        8 => {
            unsafe { crate::framework::call_game(r, game_addr(rva(RACE_CALLEES[8]))) };
            Ok(())
        }
        _ => or_in_game(RACE_CALLEES[place as usize], r, |r| race_step(place, r)),
    }
}

fn race_step(place: u32, r: &mut Regs) -> Result<(), Refused> {
    let indent = game_addr(rva(0x4E00A0)) as u32; // g_logIndent
    unsafe {
        *r = match place {
            2 => crate::race::settings_copy::copy_settings_in(
                r,
                &mut crate::race::settings_copy::game_part,
            )?,
            3 => crate::race::race_cars::set_up_arcade_cars(
                r,
                indent,
                &mut crate::race::race_cars::cars_part,
            )?,
            4 => crate::race::race_shell::set_start_states(r)?,
            5 => crate::race::stage_loading::load_stage(
                r,
                indent,
                &mut crate::race::stage_loading::game_part,
            )?,
            6 => record_replay_header(r, &mut replay_log_part)?,
            7 => crate::race::replay::open_replay(r, &mut crate::race::replay::game_part)?,
            9 => crate::race::replay::close_replay(r, &mut crate::race::replay::game_part)?,
            10 => tidy_everything(r, &LoadVars::game(), &mut tidy_part)?,
            _ => crate::race::settings_copy::copy_settings_out(
                r,
                &mut crate::race::settings_copy::game_part,
            )?,
        };
    }
    Ok(())
}

/// The plain variables `run_race` writes.
pub fn race_regions() -> Vec<Region> {
    [
        0x60101Cusize,
        0x60519C,
        0x621514,
        0x5C5CE0,
        0x6238AC,
        0x63DB84,
        0x75AFF4,
        0x75ACD8,
    ]
    .iter()
    .map(|&a| at(rva(a)))
    .collect()
}

// ---- above the race ----

/// `race::release_textures` (0x443396): `race::release_stage_textures`,
/// `scene::reset_texture_tables`. Leaves the registers as they do.
pub unsafe fn release_textures(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    part(1, &mut g)?;
    part(2, &mut g)?;
    Ok(g)
}

pub fn release_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    unsafe {
        if place == 1 {
            *r = crate::race::race_shell::release_stage_textures(
                r,
                &mut crate::race::race_shell::game_release_textures,
            );
        } else {
            crate::scene::reset_texture_tables::reset_texture_tables().apply(r);
        }
    }
    Ok(())
}

/// `race::start_race` (0x4278B7): `dword_621870` 0, `race::run_race`. Leaves the
/// registers as it does.
pub unsafe fn start_race(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    unsafe { game_write(rva(0x621870), 0u32) };
    part(1, &mut g)?;
    Ok(g)
}

pub fn start_part(_: u32, r: &mut Regs) -> Result<(), Refused> {
    let indent = game_addr(rva(0x4E00A0)) as u32; // g_logIndent
    or_in_game(0x443F0B, r, |r| {
        *r = unsafe { run_race(r, indent, &mut race_part) }?;
        Ok(())
    })
}

/// `race::race_from_menu` (0x485570): `race::render3d_tidy`, `race::objects_tidy`,
/// `nullsub_6`, `race::release_textures`, `race::start_race`. Leaves the registers as
/// the last.
pub unsafe fn race_from_menu(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut g = *entry;
    for place in [1, 2, 3, 4] {
        part(place, &mut g)?;
    }
    Ok(g)
}

pub fn menu_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::race::stage_free::{OBJECT_POINTS, POINT_RECORDS, free};
    let address = [0, 0x4433A7, 0x44346B, 0x443396, 0x4278B7][place as usize];
    or_in_game(address, r, |r| unsafe {
        match place {
            1 | 2 => {
                let g = free(
                    r,
                    if place == 1 {
                        &POINT_RECORDS
                    } else {
                        &OBJECT_POINTS
                    },
                )?;
                r.eax = g.eax;
                r.ebx = g.ebx;
                r.set_carry(false);
            }
            3 => *r = release_textures(r, &mut release_part)?,
            _ => *r = start_race(r, &mut start_part)?,
        }
        Ok(())
    })
}
