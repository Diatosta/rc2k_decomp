//! `menu::screen_478eb2_loop` (0x478EB2): the loop of a screen whose background is `control.png`.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `control.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is not asked
//! about; its own 3D list goes to `g_sceneList` before the first frame only.
//! Which screen it is has not been confirmed, hence the name. One of eight such loops in a row
//! (0x47814C .. 0x479410), by the picture the pages of the controls set-up.
//! Callers: none in code: it is reached through a table. Ungated thunk; shadow mode runs the
//! original alone; each real call
//! writes a line to hook.log. Self-test: `screen_loop::check_screen`.
//! Manual patch with a private copy (0x2B4 bytes; 12 calls and 1 jump to a fatal-error stub
//! relocated): a call or a
//! jump target sits in the first bytes.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::screen_loop::{List, Original, Screen, Step, check_screen, dispatch_screen};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::screen_478eb2_loop", rva(0x478EB2), thunk, &ORIGINAL)
        .private_copy(0x2B4)
        .relocate_calls(&[
            0x1, 0xB, 0x7C, 0xAD, 0xB2, 0xBE, 0xC3, 0x112, 0x117, 0x15F, 0x1BD, 0x25F, 0x2AD,
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
    menu: rva(0x74AA90),
    saved_item: rva(0x74773C),
    lists: &[
        List {
            pointer: rva(0x7554F4), // g_unreadList
            table: rva(0x758B72),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758DBE),
            run: false,
        },
        List {
            pointer: rva(0x755500), // dword_755500
            table: rva(0x75949E),
            run: true,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759272),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x7590AE),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758DBE),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759272),
            run: false,
        },
    ]),
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[
        Step::Call(rva(0x46B47C)),
        Step::Eax(0x0),
        Step::Call(rva(0x40939F)),
    ],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::EaxAddr(rva(0x73228C)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Call(rva(0x46B4B4))],
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
    function: rva(0x478EB2),
    len: 0x2B4,
    entry_bytes: &[0x60, 0xE8, 0xC4, 0x25, 0xFF, 0xFF, 0xB8, 0x00],
    calls: &[
        0x1, 0xB, 0x7C, 0xAD, 0xBE, 0xC3, 0x112, 0x117, 0x15F, 0x1BD, 0x25F, 0x2AD,
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
