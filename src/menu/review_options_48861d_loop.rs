//! `menu::review_options_48861d_loop` (0x48861D): the menu after the results screens, by its
//! title "OPÇÕES DE REVISÃO" (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `roptions.png`; leaving is asked about; also
//! when `dword_755569` is 1; the steps that are more than a call or a store are the functions
//! below, generated from the listing.
//! The name is a reading from the title string of its menu, "OPÇÕES DE REVISÃO", not yet a
//! visit.
//! Callers: `sub_487E2C`. Ungated thunk; shadow mode runs the original alone; each real call
//! writes a line to hook.log. Self-test: `screen_loop::check_screen`.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::screen_loop::{
    Env, List, Original, Screen, Step, check_screen, dispatch_screen, screen_entry,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::review_options_48861d_loop",
    rva(0x48861D),
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

unsafe fn set_up_first(g: &mut Regs, env: &mut dyn Env) {
    unsafe {
        game_write(rva(0x755569), 0x0u32);
        game_write(rva(0x623FB0), 0xFFFFFF64u32);
        game_write(rva(0x623FB4), 0x27u32);
        game_write(rva(0x624080), 0x140u32);
        env.call(rva(0x46F7FD), g);
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0x10u32);
        env.call(rva(0x4904DD), g);
        g.eax = game_read(rva(0x7477CC));
        game_write(rva(0x755525), g.eax);
        g.eax = g.eax.wrapping_add(0xFFFFFFFF);
        game_write(rva(0x75AA7C), g.eax);
        game_write(rva(0x7554EC), 0x0u32);
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x7512A0),
    saved_item: rva(0x7477CC),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758F9E),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758EC2),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7592F2),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[
        Step::Write(rva(0x755565), 0x0),
        Step::Write(rva(0x75555D), 0x0),
        Step::Write(rva(0x755980), 0x0),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::EaxAddr(rva(0x7329C8)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[],
    confirm: Some(&[]),
    ask_also_if: Some(rva(0x755569)),
    confirmed: &[],
    unasked_skips_save: false,
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
    also_writes: &[
        rva(0x623FB0),
        rva(0x623FB4),
        rva(0x624080),
        rva(0x7554EC),
        rva(0x75555D),
        rva(0x755565),
        rva(0x755569),
        rva(0x755980),
        rva(0x75AA74),
        rva(0x75AA7C),
    ],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_screen(&HOOK, r, call_original, &SCREEN);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    screen_entry("menu::review_options_48861d_loop", r, &SCREEN)
}

// ---- self-test ----

const THE_ORIGINAL: Original = Original {
    function: rva(0x48861D),
    len: 0x33B,
    entry_bytes: &[0x60, 0xC7, 0x05, 0x65, 0x55, 0x75, 0x00, 0x00],
    calls: &[
        0x47, 0x60, 0x10B, 0x11C, 0x121, 0x170, 0x175, 0x1BD, 0x21B, 0x2BD, 0x2EB,
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
