//! `race::load_stage` (0x444040, IDA 2026-10-17): "SHAUNINITEVERYTHING": the loading
//! picture (unless `dword_71C3E0`), the cars' sounds woken, `race::loadlevel_init`, the
//! in-game music, the controls, race init (`sub_4462B9`, the game's until ported), the
//! clock restarted and the race's starting values; each step logged and indented.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};

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

/// `race::load_stage` (0x444040): a line break, a rule, "START", "loading screen" (indent
/// in): the loading picture (`menu::show_loading_picture`) unless `dword_71C3E0`,
/// `dword_70F3E8` -1, `race::wake_car_sounds`; then each with its word and the indent round
/// it: `loadlevel_init`, the music (`game_music` with `dword_64BB18`, else
/// `game_music_8`), `create_controls`, race init (`drive::race_init`) and
/// `scene::restart_clock`; the starting values (`STARTS` with the clock, the blocks'
/// +0xFB4 3 and +0xFB0 8, `LAST_STARTS`); "FINIS", a rule, a line break. The registers
/// and flags it leaves (eax the clock, carry clear) are read by none of its seven callers
/// (`taint.py` at each site: no read point; conversion phase 3, 2026-10-24).
pub unsafe fn load_stage() -> Result<(), Refused> {
    use crate::logfile::log_write::log_write;
    let indent = |by: i32| unsafe {
        let at = crate::logfile::write_line::indent_place() as *mut u32; // g_logIndent
        at.write_unaligned(at.read_unaligned().wrapping_add(by as u32));
    };
    let mut word = 0;
    let mut log = || {
        unsafe { log_write(game_addr(rva(WORDS[word])) as *const u8, true, 1) };
        word += 1;
    };
    let newline = || unsafe { log_write(game_addr(EMPTY_TEXT_RVA) as *const u8, true, 1) };
    unsafe {
        newline();
        log();
        log();
        log();
        indent(3);
        if game_read::<u32>(REPLAY_RVA) == 0 {
            // As `menu::show_loading_picture`.
            crate::gfx::show_picture_file::show_picture_file(
                game_addr(LOADING_PICTURE_RVA) as *mut u8
            )?;
        }
        game_write(CAR_MINUS_4_RVA, u32::MAX);
        crate::race::sounds::wake_car_sounds()?;
        indent(-3);
        log();
        indent(3);
        crate::race::loadlevel::loadlevel_init()?;
        indent(-3);
        log();
        indent(3);
        if game_read::<u32>(NO_PLACE_RVA) != 0 {
            crate::race::music::game_music();
        } else {
            crate::race::music::game_music_8();
        }
        indent(-3);
        log();
        indent(3);
        crate::race::create_controls::create_controls();
        indent(-3);
        log();
        indent(3);
        crate::drive::race_init::race_init()?;
        crate::scene::restart_clock::restart_clock(crate::system::tick_count::tick_count());
        let clock: u32 = game_read(CLOCK_RVA);
        for (at, value) in STARTS {
            put(at, value.unwrap_or(clock));
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
        indent(-3);
        log();
        log();
        newline();
    }
    Ok(())
}
