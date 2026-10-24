//! `menu::start_rally_loop` (0x475C12): the mode menu, titled "COMEÇAR RALI".
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `sral.png`; leaving is not asked about; the
//! next frame begins with the steps after the first frame; it has steps of its own first on
//! leaving; the quit switch is not looked at; the steps that are more than a call or a store
//! are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "COMEÇAR RALI", not yet a visit.
//! Callers: none in code: it is reached through a table.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{
    Carried, Flow, List, Screen, Step, call, carousel, prepare_scene, screen_entry,
};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0x1u32);
        carousel(c)?;
        c.arg = game_read(rva(0x7476C4));
        game_write(rva(0x755525), c.arg);
        c.arg = c.arg.wrapping_sub(1);
        game_write(rva(0x75AA7C), c.arg);
        game_write(rva(0x623FB0), 0x7u32);
        game_write(rva(0x623FB4), 0xAu32);
        game_write(rva(0x624080), 0x100u32);
        prepare_scene(c);
    }
    Ok(())
}

unsafe fn after_first_frame(_c: &mut Carried) -> Result<Flow, Refused> {
    unsafe {
        if game_read::<u32>(rva(0x758806)) == 0x1 {
            game_write(rva(0x758806), 0x0u32);
            return Ok(Flow::SetUp);
        }
        Ok(Flow::On)
    }
}

unsafe fn leaving(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        if game_read::<u32>(rva(0x758806)) == 0x1 {
            call(rva(0x471C82), c)?;
        }
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74E2E0),
    saved_item: rva(0x7476C4),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758EEE),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758B82),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x75912A),
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
        Step::Write(rva(0x7558D8), 0x1),
        Step::ArgAddr(rva(0x731C9F)),
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
    next_frame_repeats_first: true,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: true,
    before_asking: &[],
    ask_unless: None,
    leaving: &[Step::Do(leaving)],
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
    screen_entry("menu::start_rally_loop", &SCREEN)
}
