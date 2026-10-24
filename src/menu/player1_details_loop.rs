//! `menu::player1_details_loop` (0x477692): the loop of the first player's details screen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `pdet1.png`; leaving is not asked about; its
//! own 3D list goes to `g_sceneList` before the first frame only; it has a step of its own once
//! a frame; the steps that are more than a call or a store are the functions below, generated
//! from the listing.
//! The name is a reading from the title string of its menu, "PORMENORES DO JOGADOR 1", not yet
//! a visit.
//! Callers: none in code: it is reached through a table.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, rva};
use crate::menu::screen_loop::{Carried, Flow, List, Screen, Step, call, screen_entry};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn each_frame(c: &mut Carried) -> Result<Flow, Refused> {
    unsafe {
        if game_read::<u32>(rva(0x7556A8)) == 0x1 {
            call(rva(0x467C63), c)?;
            if game_read::<u32>(rva(0x7556A8)) != 0x0 {
                return Ok(Flow::NextFrame);
            }
        }
        Ok(Flow::On)
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74E9D4),
    saved_item: rva(0x7476AC),
    lists: &[
        List {
            pointer: rva(0x7554F4), // g_unreadList
            table: rva(0x758B72),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758CBA),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591EA),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x75900E),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758CBA),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591EA),
            run: false,
        },
    ]),
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[
        Step::Write(rva(0x755980), 0x0),
        Step::Write(rva(0x75598C), 0x0),
    ],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::Write(rva(0x7558D8), 0x1),
        Step::ArgAddr(rva(0x731CE3)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
    ],
    after_first_frame: &[Step::Write(rva(0x75598C), 0x0)],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
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
};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::player1_details_loop", &SCREEN)
}
