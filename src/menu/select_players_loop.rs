//! `menu::select_players_loop` (0x48BCDC): the loop of the screen where the number of players
//! is chosen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `arcade1.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "ESCOLHA NÚMERO DE JOGADORES", not
//! yet a visit.
//! Callers: none in code: it is reached through a table. Ungated thunk; shadow mode runs the
//! original alone; each real call writes a line to hook.log. Self-test:
//! `screen_loop::check_screen`.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::screen_loop::{Env, List, Original, Screen, Step, check_screen, dispatch_screen};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::select_players_loop", rva(0x48BCDC), thunk, &ORIGINAL).self_test(self_test);

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
        game_write(rva(0x623FB0), 0x7u32);
        game_write(rva(0x623FB4), 0xAu32);
        game_write(rva(0x624080), 0x100u32);
        env.call(rva(0x46F7FD), g);
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0x2u32);
        env.call(rva(0x4904DD), g);
        g.eax = game_read(rva(0x747674));
        game_write(rva(0x755525), g.eax);
        g.eax = g.eax.wrapping_add(0xFFFFFFFF);
        game_write(rva(0x75AA7C), g.eax);
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x748858),
    saved_item: rva(0x747674),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758F3E),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758C02),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759182),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Write(rva(0x755980), 0x0)],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    background: &[
        Step::EaxAddr(rva(0x73297D)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[],
    confirm: None,
    after: &[Step::Write(rva(0x7556A4), 0x6F)],
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
    function: rva(0x48BCDC),
    len: 0x2EF,
    entry_bytes: &[0x60, 0xC7, 0x05, 0x80, 0x59, 0x75, 0x00, 0x00],
    calls: &[
        0x29, 0x42, 0xE3, 0xF4, 0xF9, 0x148, 0x14D, 0x195, 0x1F3, 0x295,
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
