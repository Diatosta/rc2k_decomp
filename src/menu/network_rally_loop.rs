//! `menu::network_rally_loop` (0x475F1A): the network rally menu, by its title "RALI EM REDE"
//! (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `nral.png`; leaving is not asked about; the
//! quit switch is not looked at; the steps that are more than a call or a store are the
//! functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "RALI EM REDE", not yet a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{Callee, Carried, List, Screen, Step, carousel, screen_entry};
use crate::menu::state_store::MENU_STATE;

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        game_write(rva(0x7554EC), 0x1u32);
        MENU_STATE.carousel.menu.set(0x6u32);
        carousel(c)?;
        c.arg = game_read(rva(0x7476A0));
        game_write(rva(0x755525), c.arg);
        c.arg = c.arg.wrapping_sub(1);
        MENU_STATE.carousel.front.set(c.arg);
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x749118),
    saved_item: rva(0x7476A0),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758F56),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758C26),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x75919A),
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
        Step::ArgAddr(rva(0x731CC1)),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
    ],
    after_first_frame: &[
        Step::Write(rva(0x623FB0), 0x7),
        Step::Write(rva(0x623FB4), 0xA),
        Step::Write(rva(0x624080), 0x100),
        Step::Call(Callee::PrepareScene),
    ],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[
        Step::Write(rva(0x758806), 0x0),
        Step::Write(rva(0x755569), 0x0),
        Step::Write(rva(0x755A3C), 0x0),
        Step::Write(rva(0x7556A4), 0x6F),
        Step::Write(rva(0x755561), 0x0),
    ],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: false,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: true,
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

/// The Rust version where the typed tables run it (a refusal is the
/// caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::network_rally_loop", &SCREEN)
}
