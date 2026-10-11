//! `menu::time_trial_stage_details_loop` (0x48F1F4): the loop of the stage details screen of a
//! time trial.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: leaving is not asked about.
//! A reading, not yet a visit: its draw list 0x758CFE is the one of
//! `screens::draw_time_trial_stage_info`, a confirmed screen; it sets `dword_755700` to 1 first.
//! Callers: `sub_47037E`. Ungated thunk; shadow mode runs the original alone; each real call
//! writes a line to hook.log. Self-test: `screen_loop::check_screen`.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::screen_loop::{List, Original, Screen, Step, check_screen, dispatch_screen};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::time_trial_stage_details_loop",
    rva(0x48F1F4),
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
    menu: rva(0x754C24),
    saved_item: rva(0x7476EC),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x759036),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758CFE),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x75920A),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Write(rva(0x755700), 0x1)],
    set_up_first: &[],
    prepare: &[],
    background: &[Step::Call(rva(0x4683CF))],
    after_first_frame: &[],
    confirm: None,
    after: &[],
    leave_at_once: None,
    also_writes: &[],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_screen(&HOOK, r, call_original, &SCREEN);
}

// ---- self-test ----

const THE_ORIGINAL: Original = Original {
    function: rva(0x48F1F4),
    len: 0x28D,
    entry_bytes: &[0x60, 0xC7, 0x05, 0x00, 0x57, 0x75, 0x00, 0x01],
    calls: &[0x91, 0x9C, 0xA1, 0xF0, 0xF5, 0x13D, 0x19B, 0x23D],
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
