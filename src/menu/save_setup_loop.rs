//! `menu::save_setup_loop` (0x46EB7A): the loop of the screen where a car setup is saved and
//! its name typed.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is not asked about; it has a step of its own once
//! a frame; the steps that are more than a call or a store are the functions below, generated
//! from the listing.
//! The name is a reading from the title string of its menu, "{}", not yet a visit.
//! Callers: none in code: it is reached through a table. Ungated thunk; shadow mode runs the
//! original alone; each real call writes a line to hook.log. Self-test:
//! `screen_loop::check_screen`. Manual patch with a private copy (0x307 bytes, 9 calls and
//! jumps relocated): a call or a jump target sits in the first bytes.

use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::screen_loop::{
    Env, Flow, List, Original, Screen, Step, check_screen, dispatch_screen,
};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::save_setup_loop", rva(0x46EB7A), thunk, &ORIGINAL)
    .private_copy(0x307)
    .relocate_calls(&[0x6B, 0xE6, 0xEB, 0x13A, 0x13F, 0x187, 0x1A5, 0x20B, 0x2AD])
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

unsafe fn prepare(g: &mut Regs, env: &mut dyn Env) {
    unsafe {
        game_write(rva(0x623FB0), 0x0u32);
        game_write(rva(0x623FB4), 0x0u32);
        game_write(rva(0x624080), 0x100u32);
        env.call(rva(0x46F7FD), g);
        game_write(rva(0x6241DC), 0x0u32);
        game_write(rva(0x6241E0), 0x320u32);
        game_write(rva(0x6241EC), game_read::<i32>(rva(0x6241DC)) as f32);
        game_write(rva(0x6241F0), game_read::<i32>(rva(0x6241E0)) as f32);
    }
}

unsafe fn each_frame(g: &mut Regs, env: &mut dyn Env) -> Flow {
    unsafe {
        if game_read::<u32>(rva(0x7556A8)) == 0x1 {
            env.call(rva(0x46F1A9), g);
            if game_read::<u32>(rva(0x7556A8)) != 0x0 {
                return Flow::NextFrame;
            }
            game_write(rva(0x755A38), 0x2u32);
            return Flow::Leave;
        }
        Flow::On
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x75296C),
    saved_item: rva(0x747678),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758F12),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758BB6),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759156),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Write(rva(0x7554E4), 0x0)],
    prepare: &[Step::Do(prepare)],
    after_lists: &[],
    background: &[],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Write(rva(0x7554E4), 0x1)],
    escape_leaves: false,
    each_frame: &[Step::DoFlow(each_frame)],
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
        rva(0x6241DC),
        rva(0x6241E0),
        rva(0x6241EC),
        rva(0x6241F0),
        rva(0x7554E4),
        rva(0x755A38),
    ],
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_screen(&HOOK, r, call_original, &SCREEN);
}

// ---- self-test ----

const THE_ORIGINAL: Original = Original {
    function: rva(0x46EB7A),
    len: 0x307,
    entry_bytes: &[0x60, 0xC7, 0x05, 0xE4, 0x54, 0x75, 0x00, 0x00],
    calls: &[0x6B, 0xE6, 0xEB, 0x13A, 0x13F, 0x187, 0x1A5, 0x20B, 0x2AD],
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
