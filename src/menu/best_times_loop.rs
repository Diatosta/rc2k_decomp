//! `menu::best_times_loop` (0x4A3EF3): the loop of the best times screen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `btimes.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is asked about.
//! A reading from the title string of its menu, not yet a visit: "MELHORES TEMPOS".
//! Callers: `sub_4A0E60`, `sub_4AA4D7`. Ungated thunk; shadow mode runs the original alone;
//! each real call
//! writes a line to hook.log. Self-test: `screen_loop::check_screen`.
//! Manual patch with a private copy (0x2BB bytes; 10 calls and 1 jump to a fatal-error stub
//! relocated): a call or a
//! jump target sits in the first bytes.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::screen_loop::{List, Original, Screen, Step, check_screen, dispatch_screen};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::best_times_loop", rva(0x4A3EF3), thunk, &ORIGINAL)
    .private_copy(0x2BB)
    .relocate_calls(&[
        0x6D, 0x9E, 0xA3, 0xAF, 0xB4, 0x103, 0x108, 0x150, 0x1AE, 0x250, 0x275,
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
    menu: rva(0x74DF10),
    saved_item: rva(0x7476FC),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758FD2),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758D1A),
            run: false,
        },
        List {
            pointer: rva(0x755500), // dword_755500
            table: rva(0x75931A),
            run: true,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759216),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[],
    prepare: &[],
    background: &[
        Step::EaxAddr(rva(0x731FCF)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[],
    confirm: Some(&[]),
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
    function: rva(0x4A3EF3),
    len: 0x2BB,
    entry_bytes: &[0x60, 0xC7, 0x05, 0x06, 0x88, 0x75, 0x00, 0x00],
    calls: &[
        0x6D, 0x9E, 0xAF, 0xB4, 0x103, 0x108, 0x150, 0x1AE, 0x250, 0x275,
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
