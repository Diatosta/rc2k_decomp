//! `menu::visual_options_loop` (0x480592): the loop of the visual options screen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `visual.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is not asked
//! about.
//! A reading from the title string of its menu, not yet a visit: "OPÇÕES VISUAIS".
//! Callers: none in code: it is reached through a table. Ungated thunk; shadow mode runs the
//! original alone; each real call
//! writes a line to hook.log. Self-test: `screen_loop::check_screen`.
//! Manual patch with a private copy (0x28F bytes; 8 calls and 1 jump to a fatal-error stub
//! relocated): a call or a
//! jump target sits in the first bytes.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::screen_loop::{List, Original, Screen, Step, check_screen, dispatch_screen};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::visual_options_loop", rva(0x480592), thunk, &ORIGINAL)
        .private_copy(0x28F)
        .relocate_calls(&[0x8D, 0x92, 0x9E, 0xA3, 0xF2, 0xF7, 0x13F, 0x19D, 0x23F])
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
    menu: rva(0x754538),
    saved_item: rva(0x74771C),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x75900A),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758CB2),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591E6),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::EaxAddr(rva(0x731E1D)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[],
    confirm: None,
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

// ---- self-test ----

const THE_ORIGINAL: Original = Original {
    function: rva(0x480592),
    len: 0x28F,
    entry_bytes: &[0x60, 0xC7, 0x05, 0x06, 0x88, 0x75, 0x00, 0x00],
    calls: &[0x8D, 0x9E, 0xA3, 0xF2, 0xF7, 0x13F, 0x19D, 0x23F],
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
