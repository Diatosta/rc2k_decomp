//! `menu::visual_preferences_loop` (0x47A22C): the loop of the visual preferences screen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `visual.png`; leaving is not asked about; its
//! own 3D list goes to `g_sceneList` before the first frame only; the steps that are more than
//! a call or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "PREFERÊNCIAS VISUAIS", not yet a
//! visit.
//! Callers: none in code: it is reached through a table. Ungated thunk; shadow mode runs the
//! original alone; each real call writes a line to hook.log. Self-test:
//! `screen_loop::check_screen`.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::screen_loop::{Env, List, Original, Screen, Step, check_screen, dispatch_screen};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::visual_preferences_loop",
    rva(0x47A22C),
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

unsafe fn before(g: &mut Regs, _env: &mut dyn Env) {
    unsafe {
        g.ecx = game_read(rva(0x7478B4));
        game_write(rva(0x747718), g.ecx);
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x748280),
    saved_item: rva(0x747718),
    lists: &[
        List {
            pointer: rva(0x7554F4), // g_unreadList
            table: rva(0x758B72),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758CAA),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591E2),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x759006),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758CAA),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591E2),
            run: false,
        },
    ]),
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[],
    prepare: &[],
    background: &[
        Step::EaxAddr(rva(0x731DF9)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
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
    function: rva(0x47A22C),
    len: 0x29B,
    entry_bytes: &[0x60, 0x8B, 0x0D, 0xB4, 0x78, 0x74, 0x00, 0x89],
    calls: &[0x99, 0xAA, 0xAF, 0xFE, 0x103, 0x14B, 0x1A9, 0x24B],
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
