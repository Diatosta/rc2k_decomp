//! `menu::select_stage_loop` (0x482099): the stage selection "ESCOLHA ETAPA" of the start menus
//! under sub_482382.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; left without the question, the
//! highlight is not saved; the next frame begins with the steps after the first frame; the
//! question is not put while `dword_52A910` is 1; the steps that are more than a call or a
//! store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "ESCOLHA ETAPA", not yet a visit.
//! Callers: `sub_418347`, `sub_482382`, `sub_4824C1`, `sub_4826A2`, `sub_4828B0`, `sub_482B7A`.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_read, game_write, rva};
use crate::menu::screen_loop::{Env, Flow, List, Screen, Step, screen_entry};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn after_first_frame(_g: &mut Regs, _env: &mut dyn Env) -> Flow {
    unsafe {
        if game_read::<u32>(rva(0x758806)) == 0x1 {
            game_write(rva(0x758806), 0x0u32);
            return Flow::SetUp;
        }
        Flow::On
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x748620),
    saved_item: rva(0x747660),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x759052),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758E9E),
            run: false,
        },
        List {
            pointer: rva(0x755500), // dword_755500
            table: rva(0x75931E),
            run: true,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7592D6),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Write(rva(0x75524C), 0x0)],
    set_up_first: &[Step::Call(rva(0x486C21))],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(rva(0x4683CF))],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: true,
    after: &[Step::Write(rva(0x7556A4), 0x6F)],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: true,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: false,
    before_asking: &[],
    ask_unless: Some(rva(0x52A910)),
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
    also_writes: &[rva(0x75524C)],
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    screen_entry("menu::select_stage_loop", r, &SCREEN)
}
