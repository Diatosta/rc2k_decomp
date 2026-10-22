//! `menu::inspection_area_loop` (0x47DBF3): the loop of the inspection area screen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `service.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is asked about.
//! A reading from the title string of its menu, not yet a visit: "ÁREA DE INSPECÇÃO".
//! Its frames after the first run with the callback `sub_46D019` instead of
//! `menu::frame_callback`: the only screen loop with two (the first build missed it and the
//! self-test caught it, 64 of 68 cases, 2026-10-11).
//! Callers: `sub_48E093`, `sub_4A15E3`, `4A1EC7`, `sub_4A8556`. Ungated thunk; shadow mode runs
//! the original alone; each real call
//! writes a line to hook.log. Self-test: `screen_loop::check_screen`.
//! Manual patch with a private copy (0x2AF bytes; 10 calls and 1 jump to a fatal-error stub
//! relocated): a call or a
//! jump target sits in the first bytes.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::screen_loop::{
    List, Original, Screen, Step, check_screen, dispatch_screen, screen_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::inspection_area_loop",
    rva(0x47DBF3),
    thunk,
    &ORIGINAL,
)
.private_copy(0x2AF)
.relocate_calls(&[
    0x8D, 0x92, 0x97, 0xA3, 0xA8, 0xF7, 0xFC, 0x144, 0x1A2, 0x244, 0x269,
])
.manual_patch()
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: rva(0x75065C),
    saved_item: rva(0x7477AC),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x7590BA),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758DDE),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x75927E),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: Some(rva(0x46D019)), // sub_46D019
    before: &[],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::EaxAddr(rva(0x7325F8)),
        Step::Call(rva(0x411863)),
        Step::CallOrFatal(rva(0x406126), 0x80712345),
    ],
    after_first_frame: &[],
    confirm: Some(&[]),
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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    screen_entry("menu::inspection_area_loop", r, &SCREEN)
}

// ---- self-test ----

const THE_ORIGINAL: Original = Original {
    function: rva(0x47DBF3),
    len: 0x2AF,
    entry_bytes: &[0x60, 0xC7, 0x05, 0x06, 0x88, 0x75, 0x00, 0x00],
    calls: &[
        0x8D, 0x92, 0xA3, 0xA8, 0xF7, 0xFC, 0x144, 0x1A2, 0x244, 0x269,
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
