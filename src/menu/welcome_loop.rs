//! `menu::welcome_loop` (0x4721F0): the screen titled "BEM-VINDO" that the championship's event
//! loops show.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; something runs before the
//! question is put.
//! The name is a reading from the title string of its menu, "BEM-VINDO", not yet a visit.
//! Callers: `sub_4A0E60`, `sub_4AA4D7`. Ungated thunk; shadow mode runs the original alone;
//! each real call writes a line to hook.log. Self-test: `screen_loop::check_screen`.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::screen_loop::{List, Original, Screen, Step, check_screen, dispatch_screen};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::welcome_loop", rva(0x4721F0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: rva(0x7506C0),
    saved_item: rva(0x7477B4),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x75905A),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758D4E),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759232),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Write(rva(0x755A3C), 0x0)],
    set_up_first: &[Step::Write(rva(0x75524C), 0x0)],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(rva(0x4684BD))],
    after_first_frame: &[Step::Call(rva(0x47F40F))],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Call(rva(0x40F757))],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: false,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: false,
    before_asking: &[Step::Call(rva(0x40F757))],
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
    also_writes: &[],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_screen(&HOOK, r, call_original, &SCREEN);
}

// ---- self-test ----

const THE_ORIGINAL: Original = Original {
    function: rva(0x4721F0),
    len: 0x2C2,
    entry_bytes: &[0x60, 0xC7, 0x05, 0x3C, 0x5A, 0x75, 0x00, 0x00],
    calls: &[
        0x9B, 0xA6, 0xAB, 0xFA, 0xFF, 0x104, 0x14C, 0x1AA, 0x24C, 0x271, 0x276, 0x2BB,
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
