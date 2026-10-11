//! `menu::leave_question_loop` (0x47FA94): the loop of the "leave?" question that
//! `menu::confirm_leaving` puts.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `mainscr.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is not asked
//! about; its own 3D list goes to `g_sceneList` before the first frame only.
//! A reading, not yet a visit: its one caller is `menu::confirm_leaving`, and it sets
//! `g_askingYesNo` to 1 while it is up, which is what makes `handle_yes_no` listen.
//! Callers: `menu::confirm_leaving`. Ungated thunk; shadow mode runs the original alone; each
//! real call
//! writes a line to hook.log. Self-test: `screen_loop::check_screen`.
//! Manual patch with a private copy (0x2A3 bytes; 8 calls and 1 jump to a fatal-error stub
//! relocated): a call or a
//! jump target sits in the first bytes.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::screen_loop::{List, Original, Screen, Step, check_screen, dispatch_screen};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::leave_question_loop", rva(0x47FA94), thunk, &ORIGINAL)
        .private_copy(0x2A3)
        .relocate_calls(&[0x97, 0x9C, 0xA8, 0xAD, 0xFC, 0x101, 0x149, 0x1A7, 0x249])
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
    menu: rva(0x7544D4),
    saved_item: rva(0x7476D0),
    lists: &[
        List {
            pointer: rva(0x7554F4), // g_unreadList
            table: rva(0x758B72),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758C9E),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591DE),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x759002),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758C9E),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591DE),
            run: false,
        },
    ]),
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Write(rva(0x7554CC), 0x1)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::EaxAddr(rva(0x731BE4)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Write(rva(0x7554CC), 0x0)],
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
    function: rva(0x47FA94),
    len: 0x2A3,
    entry_bytes: &[0x60, 0xC7, 0x05, 0xCC, 0x54, 0x75, 0x00, 0x01],
    calls: &[0x97, 0xA8, 0xAD, 0xFC, 0x101, 0x149, 0x1A7, 0x249],
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
