//! `menu::review_options_47d5a5_loop` (0x47D5A5): the menu after the results screens, by its
//! title "OPÇÕES DE REVISÃO" (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `roptions.png`; leaving is asked about; also
//! when `dword_755569` is 1; the Escape flag is looked at at the top of every frame; the steps
//! that are more than a call or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "OPÇÕES DE REVISÃO", not yet a
//! visit.
//! Callers: `sub_4A0E60`, `sub_4AA4D7`. Ungated thunk; shadow mode runs the original alone;
//! each real call writes a line to hook.log. Self-test: `screen_loop::check_screen`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_read, game_write, rva};
use crate::menu::screen_loop::{Env, List, Screen, Step, screen_entry};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(g: &mut Regs, env: &mut dyn Env) {
    unsafe {
        env.call(rva(0x471391), g);
        game_write(rva(0x755569), 0x0u32);
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0xAu32);
        env.call(rva(0x4904DD), g);
        g.eax = game_read(rva(0x74779C));
        game_write(rva(0x755525), g.eax);
        g.eax = g.eax.wrapping_add(0xFFFFFFFF);
        game_write(rva(0x75AA7C), g.eax);
        game_write(rva(0x7554EC), 0x0u32);
        game_write(rva(0x623FB0), 0xFFFFFF64u32);
        game_write(rva(0x623FB4), 0x27u32);
        game_write(rva(0x624080), 0x140u32);
        env.call(rva(0x46F7FD), g);
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x75008C),
    saved_item: rva(0x74779C),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x759062),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758D5E),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x75923A),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Write(rva(0x755A3C), 0x0)],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::EaxAddr(rva(0x73261D)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[],
    confirm: Some(&[]),
    ask_also_if: Some(rva(0x755569)),
    confirmed: &[
        Step::Write(rva(0x755569), 0x0),
        Step::Write(rva(0x758806), 0x0),
    ],
    unasked_skips_save: false,
    after: &[Step::Call(rva(0x4713DD)), Step::Write(rva(0x75594C), 0x0)],
    escape_leaves: true,
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
        rva(0x755569),
        rva(0x75594C),
        rva(0x75AA74),
        rva(0x75AA7C),
    ],
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    screen_entry("menu::review_options_47d5a5_loop", r, &SCREEN)
}
