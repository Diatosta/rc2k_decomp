//! `menu::view_courses_loop` (0x48EC30): the loop of the course information screen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is not asked about; the steps that are more than
//! a call or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "INFORMAÇÃO DO PERCURSO", not yet a
//! visit.
//! Callers: none in code: it is reached through a table. Ungated thunk; shadow mode runs the
//! original alone; each real call writes a line to hook.log. Self-test:
//! `screen_loop::check_screen`.

use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::menu::screen_loop::{
    Env, List, Original, Screen, Step, check_screen, dispatch_screen, popa, pusha,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::view_courses_loop", rva(0x48EC30), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL,
    ungated
);

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(g: &mut Regs, env: &mut dyn Env) {
    let mut all = Vec::new();
    pusha(g, &mut all);
    g.eax = game_addr(rva(0x4DEAEC)) as u32;
    env.call(rva(0x40142B), g);
    popa(g, &mut all);
}

unsafe fn after(g: &mut Regs, env: &mut dyn Env) {
    let mut all = Vec::new();
    pusha(g, &mut all);
    g.eax = game_addr(rva(0x4DEAFF)) as u32;
    env.call(rva(0x40142B), g);
    popa(g, &mut all);
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74DB68),
    saved_item: rva(0x7476E8),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x75902E),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758CF6),
            run: false,
        },
        List {
            pointer: rva(0x755500), // dword_755500
            table: rva(0x75932E),
            run: true,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759206),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[
        Step::Write(rva(0x755700), 0x1),
        Step::Write(rva(0x7558C8), 0x0),
        Step::Write(rva(0x7558CC), 0x1),
        Step::Write(rva(0x7558D0), 0x0),
        Step::Write(rva(0x7587EC), 0x0),
        Step::Write(rva(0x7587F0), 0x1),
        Step::Write(rva(0x75595C), 0x0),
        Step::Write(rva(0x755958), 0x0),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    background: &[Step::Call(rva(0x4683CF))],
    after_first_frame: &[],
    confirm: None,
    after: &[Step::Do(after)],
    leave_at_once: None,
    also_writes: &[
        rva(0x755700),
        rva(0x7558C8),
        rva(0x7558CC),
        rva(0x7558D0),
        rva(0x755958),
        rva(0x75595C),
        rva(0x7587EC),
        rva(0x7587F0),
    ],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_screen(&HOOK, r, call_original, &SCREEN);
}

// ---- self-test ----

const THE_ORIGINAL: Original = Original {
    function: rva(0x48EC30),
    len: 0x2FE,
    entry_bytes: &[0x60, 0xC7, 0x05, 0x00, 0x57, 0x75, 0x00, 0x01],
    calls: &[
        0x58, 0xCA, 0xF5, 0x100, 0x105, 0x154, 0x159, 0x1A1, 0x1FF, 0x2A1, 0x2F6,
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
