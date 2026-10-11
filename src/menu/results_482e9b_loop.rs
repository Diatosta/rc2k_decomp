//! `menu::results_482e9b_loop` (0x482E9B): a screen titled "RESULTADOS" called by sub_4828B0
//! and sub_482B7A.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `results1.png`; leaving is asked about; left
//! without the question, the highlight is not saved; its own 3D list goes to `g_sceneList`
//! before the first frame only; around an action the highlight is kept in `dword_7477D8`; the
//! action run is that of the item of another menu, 0x750ACC (the original's slip); on leaving
//! the highlight is saved in `dword_7477D8`, not in the variable it is taken from (the
//! original's slip); the keys are handled with another menu, 0x750ACC, in `dword_731BBB` (the
//! original's slip).
//! The name is a reading from the title string of its menu, "RESULTADOS", not yet a visit.
//! Callers: `sub_4828B0`, `sub_482B7A`. Ungated thunk; shadow mode runs the original alone;
//! each real call writes a line to hook.log. Self-test: `screen_loop::check_screen`.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::screen_loop::{List, Original, Screen, Step, check_screen, dispatch_screen};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::results_482e9b_loop", rva(0x482E9B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: rva(0x750BF8),
    saved_item: rva(0x7477E8),
    lists: &[
        List {
            pointer: rva(0x7554F4), // g_unreadList
            table: rva(0x758B72),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758E8E),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7592CE),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x759056),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758E8E),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7592CE),
            run: false,
        },
    ]),
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Write(rva(0x755980), 0x0)],
    set_up_first: &[Step::Call(rva(0x48340B)), Step::Call(rva(0x485CBB))],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::EaxAddr(rva(0x732AAC)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: true,
    after: &[Step::Write(rva(0x7556A4), 0x6F)],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: false,
    action_saved_item: Some(rva(0x7477D8)),
    action_menu: Some(rva(0x750ACC)),
    no_quit_check: false,
    before_asking: &[],
    ask_unless: None,
    leaving: &[],
    leave_saved_item: Some(rva(0x7477D8)),
    keys_menu: Some(rva(0x750ACC)),
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
    function: rva(0x482E9B),
    len: 0x2C8,
    entry_bytes: &[0x60, 0xC7, 0x05, 0x80, 0x59, 0x75, 0x00, 0x00],
    calls: &[
        0xB, 0x10, 0xA1, 0xB2, 0xB7, 0x106, 0x10B, 0x153, 0x1B1, 0x253, 0x278,
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
