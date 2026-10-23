//! `menu::view_courses_loop` (0x48EC30): the loop of the course information screen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is not asked about; the steps that are more than
//! a call or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "INFORMAÇÃO DO PERCURSO", not yet a
//! visit.
//! Callers: none in code: it is reached through a table.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_addr, rva};
use crate::menu::screen_loop::{Env, List, Screen, Step, popa, pusha, screen_entry};

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
    after_lists: &[],
    background: &[Step::Call(rva(0x4683CF))],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Do(after)],
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
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    screen_entry("menu::view_courses_loop", r, &SCREEN)
}
