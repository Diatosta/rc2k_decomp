//! `menu::car_specs_47af4c_loop` (0x47AF4C): a screen over the car specification background,
//! called from sub_46FD31 and sub_48AD73.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `cspecs.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from its background and callers, not yet a visit.
//! Callers: `sub_46FD31`, `sub_46FD31`, `sub_46FD31`, `sub_46FD31`, `sub_46FD31`, `sub_48AD73`.
//! Ungated thunk; shadow mode runs the original alone; each real call writes a line to
//! hook.log. Self-test: `screen_loop::check_screen`. Manual patch with a private copy (0x31C
//! bytes, 13 calls and jumps relocated): a call or a jump target sits in the first bytes.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_read, game_write, rva};
use crate::menu::screen_loop::{Env, Flow, List, Screen, Step, screen_entry};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(g: &mut Regs, env: &mut dyn Env) {
    unsafe {
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0xEu32);
        env.call(rva(0x4904DD), g);
        g.eax = game_read(rva(0x7476E0));
        game_write(rva(0x755525), g.eax);
        g.eax = g.eax.wrapping_add(0xFFFFFFFF);
        game_write(rva(0x75AA7C), g.eax);
        game_write(rva(0x7554EC), 0x0u32);
        game_write(rva(0x75AA7C), 0x0u32);
        game_write(rva(0x755990), 0x0u32);
        env.call(rva(0x4708FD), g);
        game_write(rva(0x623FB0), 0x7u32);
        game_write(rva(0x623FB4), 0xFFFFFFCEu32);
        game_write(rva(0x624080), 0x100u32);
        env.call(rva(0x46F7FD), g);
    }
}

unsafe fn after_first_frame(_g: &mut Regs, env: &mut dyn Env) -> Flow {
    if env.quitting() as u32 == 0x1 {
        return Flow::Leave;
    }
    Flow::On
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74B9D8),
    saved_item: rva(0x7476E0),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758F66),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758C42),
            run: false,
        },
        List {
            pointer: rva(0x755500), // dword_755500
            table: rva(0x75940E),
            run: true,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591AA),
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
        Step::EaxAddr(rva(0x731F3D)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
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
        rva(0x755990),
        rva(0x75AA74),
        rva(0x75AA7C),
    ],
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    screen_entry("menu::car_specs_47af4c_loop", r, &SCREEN)
}
