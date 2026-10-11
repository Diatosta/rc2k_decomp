//! `menu::new_simulation_loop` (0x4A99AA): the loop of the screen that starts a new simulation.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `sral.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "NOVA SIMULAÇÃO", not yet a visit.
//! Callers: `sub_4A96BF`. Ungated thunk; shadow mode runs the original alone; each real call
//! writes a line to hook.log. Self-test: `screen_loop::check_screen`.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::screen_loop::{Env, List, Original, Screen, Step, check_screen, dispatch_screen};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::new_simulation_loop", rva(0x4A99AA), thunk, &ORIGINAL).self_test(self_test);

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
        g.ecx = game_read(rva(0x747900));
        game_write(rva(0x747764), g.ecx);
        game_write(rva(0x755980), 0x0u32);
    }
}

unsafe fn set_up_first(g: &mut Regs, env: &mut dyn Env) {
    unsafe {
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0x2u32);
        env.call(rva(0x4904DD), g);
        g.eax = game_read(rva(0x747764));
        game_write(rva(0x755525), g.eax);
        g.eax = g.eax.wrapping_add(0xFFFFFFFF);
        game_write(rva(0x75AA7C), g.eax);
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x7496F8),
    saved_item: rva(0x747764),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758F4E),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758C1A),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759192),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    background: &[
        Step::EaxAddr(rva(0x7323F5)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[
        Step::Write(rva(0x623FB0), 0x7),
        Step::Write(rva(0x623FB4), 0xA),
        Step::Write(rva(0x624080), 0x100),
        Step::Call(rva(0x46F7FD)),
    ],
    confirm: None,
    after: &[],
    leave_at_once: None,
    also_writes: &[
        rva(0x623FB0),
        rva(0x623FB4),
        rva(0x624080),
        rva(0x7554EC),
        rva(0x755980),
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
    function: rva(0x4A99AA),
    len: 0x2F1,
    entry_bytes: &[0x60, 0x8B, 0x0D, 0x00, 0x79, 0x74, 0x00, 0x89],
    calls: &[
        0x2B, 0xCC, 0xDD, 0xE2, 0x131, 0x154, 0x159, 0x1A1, 0x1FF, 0x2A1,
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
