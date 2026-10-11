//! `menu::main_menu_loop` (0x4748D9): the loop of the main menu, the screen the game starts
//! on (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! written by hand from the disassembly, because this loop has a shape of its own:
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
//! with `shadow = false` and hook.log, 2026-10-12).
//! Caller: `sub_47486C`. Ungated thunk; shadow mode runs the original alone; each real call
//! writes a line to hook.log. Self-test: `screen_loop::check_screen`.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::screen_loop::{
    Env, Flow, List, Original, Screen, Step, UPDATE_BLINK, check_screen, dispatch_screen,
};
use crate::menu::update_frame_clock::CLOCK_RVA;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::main_menu_loop", rva(0x4748D9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- game data ----

const SAVED_ITEM_RVA: usize = rva(0x74769C); // dword_74769C
const IDLE_RVA: usize = rva(0x755551); // dword_755551: 1 after a long time without a key
const IDLE_FRAMES_RVA: usize = rva(0x755555); // dword_755555
const IDLE_HELD_RVA: usize = rva(0x5BE870); // g_noFmv: 1 = no films (`nofmv` in ral.cfg)
const BLINK_TIME_RVA: usize = rva(0x7556C4); // dword_7556C4
const SETUP_TIME_RVA: usize = rva(0x5189A0); // dword_5189A0
const FRAME_FLAG_RVA: usize = rva(0x7585D8); // dword_7585D8
const BACKGROUND_PATH_RVA: usize = rva(0x731C7A); // "files\g\graphics\screens\mainscr.png"

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(g: &mut Regs, env: &mut dyn Env) {
    unsafe {
        game_write(rva(0x755980), 0u32);
        g.eax = game_addr(BACKGROUND_PATH_RVA) as u32;
        env.call(rva(0x47EFFF), g); // menu::load_background
        game_write(rva(0x623FB0), 7u32);
        game_write(rva(0x623FB4), 0xAu32);
        game_write(rva(0x624080), 0x100u32);
        env.call(rva(0x46F7FD), g); // scene::prepare_menu_scene
        game_write(rva(0x755569), 0u32);
        env.call(rva(0x407F95), g); // input::take_key
        game_write(rva(0x755A3C), 0u32);
        game_write(rva(0x7556A4), 0x6Fu32);
        game_write(rva(0x7554EC), 1u32);
        game_write(rva(0x75AA74), 0u32);
        env.call(rva(0x4904DD), g);
        g.eax = game_read(SAVED_ITEM_RVA);
        game_write(rva(0x755525), g.eax);
        g.eax = g.eax.wrapping_sub(1);
        game_write(rva(0x75AA7C), g.eax);
    }
}

/// With the clock in eax.
unsafe fn before_first_frame(g: &mut Regs, _env: &mut dyn Env) {
    unsafe {
        game_write(BLINK_TIME_RVA, g.eax);
        game_write(FRAME_FLAG_RVA, 1u32);
    }
}

unsafe fn idle(g: &mut Regs, env: &mut dyn Env) -> Flow {
    unsafe {
        if game_read::<u32>(IDLE_RVA) == 0 {
            return Flow::On;
        }
        game_write(
            IDLE_FRAMES_RVA,
            game_read::<u32>(IDLE_FRAMES_RVA).wrapping_add(1),
        );
        if game_read::<u32>(IDLE_HELD_RVA) == 1 {
            return Flow::On;
        }
        g.eax = 1;
        env.note("menu::main_menu_loop: idle, sub_4161CD is called");
        env.call(rva(0x4161CD), g);
        env.note("menu::main_menu_loop: sub_4161CD has come back");
        game_write(IDLE_RVA, 0u32);
        env.call(UPDATE_BLINK, g);
        g.eax = game_read(CLOCK_RVA);
        game_write(BLINK_TIME_RVA, g.eax);
        game_write(SETUP_TIME_RVA, g.eax);
        Flow::SetUp
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x748CB8),
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
    also_reads: &[IDLE_RVA, IDLE_HELD_RVA],
    also_writes: &[
        rva(0x623FB0),
        rva(0x623FB4),
        rva(0x624080),
        rva(0x7554EC),
        rva(0x75AA74),
        rva(0x75AA7C),
        IDLE_FRAMES_RVA,
        BLINK_TIME_RVA,
    ],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_screen(&HOOK, r, call_original, &SCREEN);
}

// ---- self-test ----

const THE_ORIGINAL: Original = Original {
    function: rva(0x4748D9),
    len: 0x439,
    entry_bytes: &[0x60, 0xC7, 0x05, 0x5D, 0x55, 0x75, 0x00, 0x00],
    calls: &[
        0x51, 0x56, 0x5B, 0x60, 0x65, 0xA6, 0xBB, 0xDE, 0xED, 0x11A, 0x159, 0x176, 0x17B, 0x1D9,
        0x220, 0x242, 0x251, 0x2B7, 0x315, 0x379, 0x3A9, 0x3C5, 0x400, 0x405, 0x40A, 0x40F, 0x414,
    ],
};

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Err(why) = check_screen(&mut t, &SCREEN, &THE_ORIGINAL) {
        return t.skip(why);
    }
    t.finish()
}
