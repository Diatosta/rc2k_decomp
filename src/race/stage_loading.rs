//! `race::load_stage` (0x444040, IDA 2026-10-17): "SHAUNINITEVERYTHING": the loading
//! picture (unless `dword_71C3E0`), the cars' sounds woken, `race::loadlevel_init`, the
//! in-game music, the controls, race init (`sub_4462B9`, the game's until ported), the
//! clock restarted and the race's starting values; each step logged and indented.

use crate::errors::Refused;
use crate::framework::{Regs, call_game, game_addr, game_read, game_write, rva};
use crate::race::loadlevel::or_in_game;
use crate::service::harness::{Region, at};

/// A step's calls by callee (`CALLEES`' index).
pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;

/// The callees, as IDA shows them.
pub const CALLEES: [usize; 10] = [
    0x4013DE, // 0 logfile::log_newline_l1
    0x40142B, // 1 logfile::log_write_l1_nl
    0x47CEA8, // 2 menu::show_loading_picture
    0x45D255, // 3 race::wake_car_sounds
    0x44426F, // 4 race::loadlevel_init
    0x470F8A, // 5 race::game_music
    0x470378, // 6 race::game_music_8
    0x40941A, // 7 race::create_controls
    0x4462B9, // 8 sub_4462B9 (race init: the game's)
    0x450381, // 9 scene::restart_clock
];
const NEWLINE: u32 = 0;
const LOG: u32 = 1;

/// "------", "SHAUNINITEVERYTHING - START", ": loading screen / ..", ": loadlevel_init",
/// ": doingamemusic", ": createcontrols", ": race init", "- FINIS", "------".
const WORDS: [usize; 9] = [
    0x4DBD40, 0x4DBD5D, 0x4DBD79, 0x4DBDA7, 0x4DBDCC, 0x4DBDF0, 0x4DBE15, 0x4DBE35, 0x4DBE51,
];
const REPLAY_RVA: usize = crate::race::sounds::REPLAY_RVA; // dword_71C3E0
const CAR_MINUS_4_RVA: usize = rva(0x70F3E8); // dword_70F3E8
const NO_PLACE_RVA: usize = rva(0x64BB18); // dword_64BB18
const CLOCK_RVA: usize = rva(0x613830); // g_sceneClock
const EMPTY_TEXT_RVA: usize = rva(0x4E00B7); // what `log_newline_l1` writes
const LOADING_PICTURE_RVA: usize = rva(0x731BBF); // "files\g\graphics\screens\loading.pcx"
/// The starting values: (address, value; `None` the clock).
const STARTS: [(usize, Option<u32>); 7] = [
    (0x5CC3BC, None),
    (0x5CC3B8, Some(0)),
    (0x61383C, None),
    (0x613840, None),
    (0x6D1810, Some(0)),
    (0x6D1482, Some(u32::MAX)),
    (0x6D1492, Some(u32::MAX)),
];
/// Each car block's +0xFB4 (3) and +0xFB0 (8), from 0x7103A0 / 0x71039C, 0xFE0 apart.
const BLOCK_FB4: usize = 0x7103A0;
const BLOCK_FB0: usize = 0x71039C;
const LAST_STARTS: [(usize, u32); 5] = [
    (0x621480, 1),
    (0x621890, 1),
    (0x639684, 0),
    (0x63DB74, 1),
    (0x63DB78, 1),
];

unsafe fn put(at: usize, v: u32) {
    unsafe { (game_addr(rva(at)) as *mut u32).write_unaligned(v) };
}

struct Steps<'a> {
    g: Regs,
    indent: u32,
    part: Part<'a>,
    word: usize,
}

impl Steps<'_> {
    fn log(&mut self) -> Result<(), Refused> {
        let mut r = Regs {
            eax: game_addr(rva(WORDS[self.word])) as u32,
            ..self.g
        };
        self.word += 1;
        (self.part)(LOG, &mut r)
    }

    fn newline(&mut self) -> Result<(), Refused> {
        let mut r = self.g;
        (self.part)(NEWLINE, &mut r)
    }

    fn indent(&mut self, by: i32) {
        unsafe {
            let at = self.indent as *mut u32;
            at.write_unaligned(at.read_unaligned().wrapping_add(by as u32));
        }
    }

    fn call(&mut self, callee: u32) -> Result<(), Refused> {
        (self.part)(callee, &mut self.g)
    }
}

/// `race::load_stage` (0x444040): a line break, a rule, "START", "loading screen" (indent
/// in): the loading picture unless `dword_71C3E0`, `dword_70F3E8` -1, `wake_car_sounds`;
/// then each with its word and the indent round it: `loadlevel_init`, the music
/// (`game_music` with `dword_64BB18`, else `game_music_8`), `create_controls`, race init
/// and `restart_clock`; the starting values (`STARTS` with the clock, the blocks' +0xFB4
/// 3 and +0xFB0 8, `LAST_STARTS`); "FINIS", a rule, a line break. Leaves eax the clock,
/// the rest as race init and the clock leave them; carry clear. `indent` is
/// `g_logIndent`'s address (the test's own in the self-test).
pub unsafe fn load_stage(entry: &Regs, indent: u32, part: Part) -> Result<Regs, Refused> {
    let mut s = Steps {
        g: *entry,
        indent,
        part,
        word: 0,
    };
    unsafe {
        s.newline()?;
        s.log()?;
        s.log()?;
        s.log()?;
        s.indent(3);
        if game_read::<u32>(REPLAY_RVA) == 0 {
            s.call(2)?;
        }
        game_write(CAR_MINUS_4_RVA, u32::MAX);
        s.call(3)?;
        s.indent(-3);
        s.log()?;
        s.indent(3);
        s.call(4)?;
        s.indent(-3);
        s.log()?;
        s.indent(3);
        s.call(if game_read::<u32>(NO_PLACE_RVA) != 0 {
            5
        } else {
            6
        })?;
        s.indent(-3);
        s.log()?;
        s.indent(3);
        s.call(7)?;
        s.indent(-3);
        s.log()?;
        s.indent(3);
        s.call(8)?;
        s.call(9)?;
        s.g.eax = game_read(CLOCK_RVA);
        for (at, value) in STARTS {
            put(at, value.unwrap_or(s.g.eax));
        }
        for n in 0..8 {
            put(BLOCK_FB4 + n * 0xFE0, 3);
        }
        for n in 0..8 {
            put(BLOCK_FB0 + n * 0xFE0, 8);
        }
        for (at, value) in LAST_STARTS {
            put(at, value);
        }
        s.indent(-3);
        s.log()?;
        s.log()?;
        s.newline()?;
        s.g.set_carry(false);
    }
    Ok(s.g)
}

/// The calls in the game: the Rust versions (each through `or_in_game`); race init the
/// game's.
pub fn game_part(callee: u32, r: &mut Regs) -> Result<(), Refused> {
    or_in_game(CALLEES[callee as usize], r, |r| step_part(callee, r))
}

fn step_part(callee: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::logfile::log_write::log_write;
    unsafe {
        match callee {
            NEWLINE => log_write(game_addr(EMPTY_TEXT_RVA) as *const u8, true, 1),
            LOG => log_write(r.eax as *const u8, true, 1),
            2 => {
                // As `menu::show_loading_picture`'s dispatch.
                r.eax = game_addr(LOADING_PICTURE_RVA) as u32;
                crate::gfx::show_picture_file::show_picture_file(r, &mut |at, regs| {
                    crate::gfx::show_picture_file::game_call(at, regs)
                });
            }
            3 => {
                let g = crate::race::sounds::wake_car_sounds(
                    r,
                    &mut crate::race::sounds::game_wake_part,
                )?;
                r.ebx = g.ebx;
                r.edi = g.edi;
            }
            4 => {
                *r = crate::race::loadlevel::loadlevel_init(
                    r,
                    crate::race::loadlevel::LoadVars::game(),
                    &mut crate::race::loadlevel::game_part,
                )?
            }
            5 => crate::race::music::game_music(r, &mut crate::race::music::game_tune)?,
            6 => crate::race::music::game_music_8(r, &mut crate::race::music::game_tune)?,
            7 => crate::race::create_controls::create_controls(),
            8 => call_game(r, game_addr(rva(CALLEES[8]))),
            _ => {
                r.eax = crate::scene::restart_clock::restart_clock(
                    crate::system::tick_count::tick_count(),
                )
            }
        }
    }
    Ok(())
}

/// What `load_stage` writes or reads that the test keeps.
pub fn regions() -> Vec<Region> {
    let mut r: Vec<Region> = vec![
        at(REPLAY_RVA),
        at(CAR_MINUS_4_RVA),
        at(NO_PLACE_RVA),
        at(CLOCK_RVA),
    ];
    r.extend(STARTS.iter().map(|&(a, _)| (game_addr(rva(a)) as u32, 4)));
    for n in 0..8 {
        r.push(at(rva(BLOCK_FB4 + n * 0xFE0)));
        r.push(at(rva(BLOCK_FB0 + n * 0xFE0)));
    }
    r.extend(LAST_STARTS.iter().map(|&(a, _)| at(rva(a))));
    r
}

/// A case's inputs: the replay switch, `dword_64BB18`, the clock.
pub fn prepare(replay: u32, no_place: u32, clock: u32) {
    unsafe {
        for &(a, len) in &regions() {
            std::ptr::write_bytes(a as *mut u8, 0x5A, len);
        }
        game_write(REPLAY_RVA, replay);
        game_write(NO_PLACE_RVA, no_place);
        game_write(CLOCK_RVA, clock);
    }
}

/// The copy's calls redirected by target: callee n to stand-in n + 1.
pub fn redirect(target: usize) -> Option<usize> {
    CALLEES
        .iter()
        .position(|&k| rva(k) == target)
        .map(|n| crate::install::stand_ins::stand_in(n as u32 + 1))
}
