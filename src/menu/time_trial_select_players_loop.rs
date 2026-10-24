//! `menu::time_trial_select_players_loop` (0x48BCDC): the loop of the screen where the number of players
//! is chosen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `arcade1.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! Screen: the player count of the time trial, "ESCOLHA NÚMERO DE JOGADORES" (the owner
//! said so after Rust-run visits, 2026-10-11). Named `select_players_loop` until then.
//! Callers: none in code: it is reached through a table.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{
    Carried, List, Screen, Step, carousel, prepare_scene, screen_entry,
};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        game_write(rva(0x623FB0), 0x7u32);
        game_write(rva(0x623FB4), 0xAu32);
        game_write(rva(0x624080), 0x100u32);
        prepare_scene(c);
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0x2u32);
        carousel(c)?;
        c.arg = game_read(rva(0x747674));
        game_write(rva(0x755525), c.arg);
        c.arg = c.arg.wrapping_sub(1);
        game_write(rva(0x75AA7C), c.arg);
    }
    Ok(())
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
    after_lists: &[],
    background: &[
        Step::ArgAddr(rva(0x73297D)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Write(rva(0x7556A4), 0x6F)],
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
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::time_trial_select_players_loop", &SCREEN)
}
