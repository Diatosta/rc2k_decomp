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
use crate::framework::rva;
use crate::menu::captions_store::CAPTIONS;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::menus_store::MENUS;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{Callee, Carried, List, Screen, Step, carousel, screen_entry};
use crate::menu::state_store::MENU_STATE;
use crate::scene::draw_store::DRAW;

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    SCREEN_FLOW.turning.set(0x1u32);
    MENU_STATE.carousel.menu.set(0x6u32);
    carousel(c)?;
    c.arg = SCREEN_ITEMS.current[slot::NETWORK_RALLY].get();
    LOOP_FLAGS.item.set(c.arg);
    c.arg = c.arg.wrapping_sub(1);
    MENU_STATE.carousel.front.set(c.arg);

    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.network_rally.id(),
    saved_item: || &SCREEN_ITEMS.current[slot::NETWORK_RALLY],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.network_rally_scene.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.network_rally_draw.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.network_rally_overlay.id(),
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
        Step::ArgAddr(|| {
            (crate::menu::texts_store::MENU_TEXTS
                .network_rally_picture
                .addr()) as usize
        }),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
    ],
    after_first_frame: &[
        Step::Write(|| &DRAW.view_offset[0], 0x7),
        Step::Write(|| &DRAW.view_offset[1], 0xA),
        Step::Write(|| &DRAW.view_zoom, 0x100),
        Step::Call(Callee::PrepareScene),
    ],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[
        Step::Write(|| &CAPTIONS.screen_left, 0x0),
        Step::Write(|| &LOOP_FLAGS.ended_a, 0x0),
        Step::Write(|| &DRAW_WORK.escaped, 0x0),
        Step::Write(|| &OPTION_WORK.menu_sound, 0x6F),
        Step::Write(|| &LOOP_FLAGS.back, 0x0),
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
