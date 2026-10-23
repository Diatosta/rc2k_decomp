//! `menu::view_best_scores_loop` (0x48EF2E): the loop of the best scores screen opened from the
//! menus.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `bascores.png`; leaving is not asked about;
//! the steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "MELHORES CLASSIFICAÇÕES", not yet
//! a visit.
//! Callers: none in code: it is reached through a table.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_addr, game_read, game_write, rva};
use crate::menu::screen_loop::{Env, List, Screen, Step, popa, pusha, screen_entry};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn before(g: &mut Regs, _env: &mut dyn Env) {
    unsafe {
        g.ecx = game_read(rva(0x747894));
        game_write(rva(0x7476F8), g.ecx);
    }
}

unsafe fn set_up_first(g: &mut Regs, env: &mut dyn Env) {
    let mut all = Vec::new();
    pusha(g, &mut all);
    g.eax = game_addr(rva(0x4DEB10)) as u32;
    env.call(rva(0x40142B), g);
    popa(g, &mut all);
}

unsafe fn after(g: &mut Regs, env: &mut dyn Env) {
    let mut all = Vec::new();
    pusha(g, &mut all);
    g.eax = game_addr(rva(0x4DEB22)) as u32;
    env.call(rva(0x40142B), g);
    popa(g, &mut all);
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x751474),
    saved_item: rva(0x7476F8),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758FCE),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758D0E),
            run: false,
        },
        List {
            pointer: rva(0x755500), // dword_755500
            table: rva(0x759312),
            run: true,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759212),
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
    background: &[
        Step::EaxAddr(rva(0x731FA9)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
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
    screen_entry("menu::view_best_scores_loop", r, &SCREEN)
}
