//! `menu::main_menu_loop` (0x4748D9): the loop of the main menu, the screen the game starts
//! on (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! - background `mainscr.png`, loaded first thing at set-up, and its carry is not looked at;
//! - `dword_758806` is cleared and the highlight put in place only after the first frame
//!   (`late_highlight`), and `dword_7585D8` = 1 is stored before every frame;
//! - an idle timer at the top of each frame (`idle`): while `dword_755551` is not 0 (set by
//!   `menu::update_blink` after 300,000 ms without a key) `dword_755555` counts up, and
//!   unless `g_noFmv` (0x5BE870, `nofmv` in `var\ral.cfg`) is 1, `sub_4161CD` is called with
//!   eax = 1 (it plays the intro film), the flag cleared and the screen set up again;
//! - after an action: `restart_action_check`;
//! - leaving is always asked about, and after a "yes" the loop returns at once
//!   (`always_asks`); the quit switch leads to the question too.
//!
//! The five calls and three stores between the last jump and the final `popa` (0x474CD9 ..
//! 0x474D0F) are never reached and are not in the Rust version.
//! The name is a reading from the menu (0x748CB8): its items open `start_rally_loop`,
//! `load_championship_loop`, `network_rally_loop` and `options_loop`, and the fifth is
//! `menu::go_back`.
//! Screen: the main menu. With `nofmv=0` the intro film played after five minutes without a
//! key while the Rust version ran it, and with `nofmv=1` nothing happened (the owner's runs
//! with `shadow = false` and hook.log, 2026-10-12). Caller: `sub_47486C`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, log, rva};
use crate::menu::screen_loop::{Carried, Flow, List, Screen, Step, call, carousel, prepare_scene};
use crate::menu::update_blink::update_blink;
use crate::menu::update_frame_clock::CLOCK_RVA;

// ---- game data ----

const MENU_RVA: usize = rva(0x748CB8); // off_748CB8
const SAVED_ITEM_RVA: usize = rva(0x74769C); // dword_74769C
const IDLE_RVA: usize = rva(0x755551); // dword_755551: 1 after a long time without a key
const IDLE_FRAMES_RVA: usize = rva(0x755555); // dword_755555
const IDLE_HELD_RVA: usize = rva(0x5BE870); // g_noFmv: 1 = no films (`nofmv` in ral.cfg)
const BLINK_TIME_RVA: usize = rva(0x7556C4); // dword_7556C4
const SETUP_TIME_RVA: usize = rva(0x5189A0); // dword_5189A0
const FRAME_FLAG_RVA: usize = rva(0x7585D8); // dword_7585D8
const BACKGROUND_PATH_RVA: usize = rva(0x731C7A); // "files\g\graphics\screens\mainscr.png"

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        game_write(rva(0x755980), 0u32);
        c.arg = game_addr(BACKGROUND_PATH_RVA) as u32;
        call(rva(0x47EFFF), c)?; // menu::load_background (its carry not looked at)
        game_write(rva(0x623FB0), 7u32);
        game_write(rva(0x623FB4), 0xAu32);
        game_write(rva(0x624080), 0x100u32);
        prepare_scene(c); // scene::prepare_menu_scene
        game_write(rva(0x755569), 0u32);
        call(rva(0x407F95), c)?; // input::take_key
        game_write(rva(0x755A3C), 0u32);
        game_write(rva(0x7556A4), 0x6Fu32);
        game_write(rva(0x7554EC), 1u32);
        game_write(rva(0x75AA74), 0u32);
        carousel(c)?;
        c.arg = game_read(SAVED_ITEM_RVA);
        game_write(rva(0x755525), c.arg);
        c.arg = c.arg.wrapping_sub(1);
        game_write(rva(0x75AA7C), c.arg);
    }
    Ok(())
}

/// With the clock in `Carried::arg`.
unsafe fn before_first_frame(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        game_write(BLINK_TIME_RVA, c.arg);
        game_write(FRAME_FLAG_RVA, 1u32);
    }
    Ok(())
}

unsafe fn idle(c: &mut Carried) -> Result<Flow, Refused> {
    unsafe {
        if game_read::<u32>(IDLE_RVA) == 0 {
            return Ok(Flow::On);
        }
        game_write(
            IDLE_FRAMES_RVA,
            game_read::<u32>(IDLE_FRAMES_RVA).wrapping_add(1),
        );
        if game_read::<u32>(IDLE_HELD_RVA) == 1 {
            return Ok(Flow::On);
        }
        c.arg = 1;
        log("menu::main_menu_loop: idle, sub_4161CD is called");
        // `menu::play_films` with mode 1 in eax; it keeps every register. edi is the menu
        // here (the loop's last store to it); ebp it does not read.
        crate::menu::play_films::play_films(true)?;
        log("menu::main_menu_loop: sub_4161CD has come back");
        game_write(IDLE_RVA, 0u32);
        update_blink();
        c.arg = game_read(CLOCK_RVA);
        game_write(BLINK_TIME_RVA, c.arg);
        game_write(SETUP_TIME_RVA, c.arg);
        Ok(Flow::SetUp)
    }
}

pub const SCREEN: Screen = Screen {
    menu: MENU_RVA,
    saved_item: SAVED_ITEM_RVA,
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758EE6),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758B76),
            run: false,
        },
        List {
            pointer: rva(0x755500), // dword_755500
            table: rva(0x75934A),
            run: true,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759122),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[
        Step::Write(rva(0x75555D), 0),
        Step::Write(rva(0x755561), 0),
        Step::Write(rva(0x755565), 0),
        Step::Write(rva(0x755980), 0),
        Step::Write(rva(0x7556A4), 0x6F),
        Step::Write(rva(0x755A3C), 0),
        Step::Write(SAVED_ITEM_RVA, 1),
        Step::Write(rva(0x755525), 1),
        Step::Call(rva(0x47C63F)),
        Step::Call(rva(0x407F95)), // input::take_key
        Step::Call(rva(0x442AF6)),
        Step::Call(rva(0x474E04)),
        Step::Call(rva(0x442AF6)),
        Step::Write(rva(0x635084), 0x50),
        Step::Write(rva(0x635088), 0xFF),
        Step::Write(rva(0x63508C), 0),
        Step::Write(rva(0x635090), 0xFF),
        Step::Write(rva(0x635094), 0x6E),
        Step::Write(rva(0x635098), 0x1E),
        Step::Call(rva(0x40F792)),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[],
    after_first_frame: &[],
    confirm: Some(&[
        Step::Write(rva(0x755569), 0),
        Step::Write(rva(0x755A3C), 0),
        Step::Write(rva(0x7556A4), 0x6F),
        Step::Write(rva(0x755561), 0),
    ]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: false,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: false,
    before_asking: &[],
    ask_unless: None,
    leaving: &[],
    leave_saved_item: None,
    keys_menu: None,
    after_restored: &[],
    result: None,
    late_highlight: true,
    before_first_frame: &[Step::Do(before_first_frame)],
    before_frame: &[Step::Write(FRAME_FLAG_RVA, 1)],
    after_blink: &[Step::DoFlow(idle)],
    restart_action_check: true,
    always_asks: true,
};
