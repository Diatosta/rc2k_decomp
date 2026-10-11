//! `menu::car_specification_loop` (0x47A652): the screen titled "ESPECIFICAÇÕES DO CARRO".
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is not asked about; it has a step of its own once
//! a frame; one call comes after the registers are put back; the steps that are more than a
//! call or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "ESPECIFICAÇÕES DO CARRO", not yet
//! a visit.
//! Callers: none in code: it is reached through a table. Ungated thunk; shadow mode runs the
//! original alone; each real call writes a line to hook.log. Self-test:
//! `screen_loop::check_screen`.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::screen_loop::{Env, List, Original, Screen, Step, check_screen, dispatch_screen};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::car_specification_loop",
    rva(0x47A652),
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
        g.eax = game_read(rva(0x7585DC));
        game_write(rva(0x755990), g.eax);
        game_write(rva(0x755998), 0x0u32);
    }
}

unsafe fn set_up_first(g: &mut Regs, env: &mut dyn Env) {
    unsafe {
        game_write(rva(0x623FB0), 0xFFFFFF60u32);
        game_write(rva(0x623FB4), 0xFFFFFF9Cu32);
        game_write(rva(0x624080), 0x100u32);
        env.call(rva(0x46F7FD), g);
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0xBu32);
        env.call(rva(0x4904DD), g);
        g.eax = game_read(rva(0x7476D4));
        game_write(rva(0x755525), g.eax);
        g.eax = g.eax.wrapping_add(0xFFFFFFFF);
        game_write(rva(0x75AA7C), g.eax);
        game_write(rva(0x7554EC), 0x0u32);
        g.eax = game_read(rva(0x755990));
        game_write(rva(0x75AA7C), g.eax);
    }
}

unsafe fn after(g: &mut Regs, _env: &mut dyn Env) {
    unsafe {
        g.eax = game_read(rva(0x755990));
        game_write(rva(0x7585DC), g.eax);
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74B628),
    saved_item: rva(0x7476D4),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758F76),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758CDA),
            run: false,
        },
        List {
            pointer: rva(0x755500), // dword_755500
            table: rva(0x7593C6),
            run: true,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591FA),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(rva(0x468482)), Step::Call(rva(0x469AA1))],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Do(after)],
    escape_leaves: false,
    each_frame: &[Step::Call(rva(0x46F7AD))],
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
    after_restored: &[Step::Call(rva(0x40F757))],
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
        rva(0x755990),
        rva(0x755998),
        rva(0x7585DC),
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
    function: rva(0x47A652),
    len: 0x321,
    entry_bytes: &[0x60, 0xA1, 0xDC, 0x85, 0x75, 0x00, 0xA3, 0x90],
    calls: &[
        0x33, 0x4C, 0xE1, 0x10C, 0x111, 0x11C, 0x121, 0x170, 0x175, 0x1BD, 0x1D2, 0x220, 0x2C2,
        0x31B,
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
