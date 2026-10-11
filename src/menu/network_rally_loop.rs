//! `menu::network_rally_loop` (0x475F1A): the network rally menu, by its title "RALI EM REDE"
//! (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `nral.png`; leaving is not asked about; the
//! quit switch is not looked at; the steps that are more than a call or a store are the
//! functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "RALI EM REDE", not yet a visit.
//! Callers: none in code: it is reached through a table. Ungated thunk; shadow mode runs the
//! original alone; each real call writes a line to hook.log. Self-test:
//! `screen_loop::check_screen`. Manual patch with a private copy (0x30A bytes, 11 calls and
//! jumps relocated): a call or a jump target sits in the first bytes.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::screen_loop::{Env, List, Original, Screen, Step, check_screen, dispatch_screen};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::network_rally_loop", rva(0x475F1A), thunk, &ORIGINAL)
        .private_copy(0x30A)
        .relocate_calls(&[
            0x15, 0xB6, 0xBB, 0xC7, 0xCC, 0x11B, 0x13E, 0x143, 0x188, 0x1E6, 0x288,
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

unsafe fn set_up_first(g: &mut Regs, env: &mut dyn Env) {
    unsafe {
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0x6u32);
        env.call(rva(0x4904DD), g);
        g.eax = game_read(rva(0x7476A0));
        game_write(rva(0x755525), g.eax);
        g.eax = g.eax.wrapping_add(0xFFFFFFFF);
        game_write(rva(0x75AA7C), g.eax);
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x749118),
    saved_item: rva(0x7476A0),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758F56),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758C26),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x75919A),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::EaxAddr(rva(0x731CC1)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[
        Step::Write(rva(0x623FB0), 0x7),
        Step::Write(rva(0x623FB4), 0xA),
        Step::Write(rva(0x624080), 0x100),
        Step::Call(rva(0x46F7FD)),
    ],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[
        Step::Write(rva(0x758806), 0x0),
        Step::Write(rva(0x755569), 0x0),
        Step::Write(rva(0x755A3C), 0x0),
        Step::Write(rva(0x7556A4), 0x6F),
        Step::Write(rva(0x755561), 0x0),
    ],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: false,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: true,
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
        rva(0x755569),
        rva(0x75AA74),
        rva(0x75AA7C),
    ],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_screen(&HOOK, r, call_original, &SCREEN);
}

// ---- self-test ----

const THE_ORIGINAL: Original = Original {
    function: rva(0x475F1A),
    len: 0x30A,
    entry_bytes: &[0x60, 0xC7, 0x05, 0xEC, 0x54, 0x75, 0x00, 0x01],
    calls: &[
        0x15, 0xB6, 0xC7, 0xCC, 0x11B, 0x13E, 0x143, 0x188, 0x1E6, 0x288,
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
