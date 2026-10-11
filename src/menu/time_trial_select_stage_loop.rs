//! `menu::time_trial_select_stage_loop` (0x48BFCB): the stage selection of the time trial, by
//! its title "ESCOLHA ETAPA" and its callers (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; left without the question, the
//! highlight is not saved.
//! The name is a reading from the title string of its menu, "ESCOLHA ETAPA", not yet a visit.
//! Callers: `sub_48AE39`, `sub_48AFD0`, `sub_48B23D`, `sub_48B54B`, `sub_48B90F`. Ungated
//! thunk; shadow mode runs the original alone; each real call writes a line to hook.log.
//! Self-test: `screen_loop::check_screen`.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::screen_loop::{List, Original, Screen, Step, check_screen, dispatch_screen};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::time_trial_select_stage_loop",
    rva(0x48BFCB),
    thunk,
    &ORIGINAL,
)
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
    menu: rva(0x74873C),
    saved_item: rva(0x747664),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758FBE),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758EA6),
            run: false,
        },
        List {
            pointer: rva(0x755500), // dword_755500
            table: rva(0x759326),
            run: true,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7592DA),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Write(rva(0x75524C), 0x0)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[Step::Write(rva(0x755700), 0x1)],
    background: &[Step::Call(rva(0x4683CF))],
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
    function: rva(0x48BFCB),
    len: 0x2CD,
    entry_bytes: &[0x60, 0xC7, 0x05, 0x4C, 0x52, 0x75, 0x00, 0x00],
    calls: &[
        0x77, 0xAC, 0xB7, 0xBC, 0x10B, 0x110, 0x158, 0x1B6, 0x258, 0x27D,
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
