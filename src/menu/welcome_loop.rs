//! `menu::welcome_loop` (0x4721F0): the screen titled "BEM-VINDO" that the championship's event
//! loops show.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; something runs before the
//! question is put.
//! The name is a reading from the title string of its menu, "BEM-VINDO", not yet a visit.
//! Callers: `sub_4A0E60`, `sub_4AA4D7`. Ungated thunk; shadow mode runs the original alone;
//! each real call writes a line to hook.log. Self-test: `screen_loop::check_screen`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, rva};
use crate::menu::screen_loop::{List, Screen, Step, screen_entry};

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: rva(0x7506C0),
    saved_item: rva(0x7477B4),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x75905A),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758D4E),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759232),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Write(rva(0x755A3C), 0x0)],
    set_up_first: &[Step::Write(rva(0x75524C), 0x0)],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(rva(0x4684BD))],
    after_first_frame: &[Step::Call(rva(0x47F40F))],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Call(rva(0x40F757))],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: false,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: false,
    before_asking: &[Step::Call(rva(0x40F757))],
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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    screen_entry("menu::welcome_loop", r, &SCREEN)
}
