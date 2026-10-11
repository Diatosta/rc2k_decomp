//! `menu::next_stage_loop` (0x473B21): the stage details screen before a stage of a
//! championship, by its title "ETAPA SEGUINTE" and its draw list, which is that of
//! `screens::draw_stage_info` (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; it has a step of its own once a
//! frame; the steps that are more than a call or a store are the functions below, generated
//! from the listing.
//! The name is a reading from the title string of its menu, "ETAPA SEGUINTE", not yet a visit.
//! Callers: `470393`, `sub_4A0E60`, `sub_4AA4D7`. Ungated thunk; shadow mode runs the original
//! alone; each real call writes a line to hook.log. Self-test: `screen_loop::check_screen`.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::screen_loop::{Env, List, Original, Screen, Step, check_screen, dispatch_screen};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::next_stage_loop", rva(0x473B21), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn before(g: &mut Regs, _env: &mut dyn Env) {
    unsafe {
        g.ecx = game_read(rva(0x747904));
        game_write(rva(0x747768), g.ecx);
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74DA3C),
    saved_item: rva(0x747768),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x759092),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758D7E),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759256),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[Step::Write(rva(0x755700), 0x3)],
    background: &[Step::Call(rva(0x4683CF))],
    after_first_frame: &[Step::Call(rva(0x47F437))],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Call(rva(0x40F757))],
    escape_leaves: false,
    each_frame: &[Step::Call(rva(0x46F7AD))],
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
    late_highlight: false,
    before_first_frame: &[],
    before_frame: &[],
    after_blink: &[],
    restart_action_check: false,
    always_asks: false,
    also_reads: &[],
    also_writes: &[rva(0x755700)],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_screen(&HOOK, r, call_original, &SCREEN);
}

// ---- self-test ----

const THE_ORIGINAL: Original = Original {
    function: rva(0x473B21),
    len: 0x2C3,
    entry_bytes: &[0x60, 0x8B, 0x0D, 0x04, 0x79, 0x74, 0x00, 0x89],
    calls: &[
        0x9D, 0xA8, 0xAD, 0xFC, 0x101, 0x106, 0x14E, 0x163, 0x1B1, 0x253, 0x278, 0x2BC,
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
