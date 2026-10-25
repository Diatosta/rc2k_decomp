//! `race::load_stage` (0x444040, IDA 2026-10-17): "SHAUNINITEVERYTHING": the loading
//! picture (unless `dword_71C3E0`), the cars' sounds woken, `race::loadlevel_init`, the
//! in-game music, the controls, race init (`sub_4462B9`, the game's until ported), the
//! clock restarted and the race's starting values; each step logged and indented.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::physics::tables_store::PHYSICS;
use crate::race::blocks_store::CAR_BLOCKS;

/// "------", "SHAUNINITEVERYTHING - START", ": loading screen / ..", ": loadlevel_init",
/// ": doingamemusic", ": createcontrols", ": race init", "- FINIS", "------".
const WORDS: [&std::ffi::CStr; 9] = [
    c"----------------------------",
    c"SHAUNINITEVERYTHING - START",
    c"SHAUNINITEVERYTHING : loading screen / sounds",
    c"SHAUNINITEVERYTHING : loadlevel_init",
    c"SHAUNINITEVERYTHING : doingamemusic",
    c"SHAUNINITEVERYTHING : createcontrols",
    c"SHAUNINITEVERYTHING : race init",
    c"SHAUNINITEVERYTHING - FINIS",
    c"----------------------------",
];
const REPLAY_RVA: usize = crate::race::sounds::REPLAY_RVA; // dword_71C3E0
const CAR_MINUS_4_RVA: usize = rva(0x70F3E8); // dword_70F3E8
const NO_PLACE_RVA: usize = rva(0x64BB18); // dword_64BB18
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
        unsafe { log_write(WORDS[word].as_ptr() as usize as *const u8, true, 1) };
        word += 1;
    };
    let newline = || unsafe { log_write(c"".as_ptr().cast(), true, 1) };
    unsafe {
        newline();
        log();
        log();
        log();
        indent(3);
        if game_read::<u32>(REPLAY_RVA) == 0 {
            // As `menu::show_loading_picture`.
            crate::gfx::show_picture_file::show_picture_file(
                crate::menu::texts_store::MENU_TEXTS.loading_pcx.addr() as *mut u8,
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
        let clock: u32 = PHYSICS.scene_clock.bits();
        for (at, value) in STARTS {
            put(at, value.unwrap_or(clock));
        }
        for car in &CAR_BLOCKS.cars[..8] {
            car.light_hold_steps.set(3);
        }
        for car in &CAR_BLOCKS.cars[..8] {
            car.value_jump_steps.set(8);
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
