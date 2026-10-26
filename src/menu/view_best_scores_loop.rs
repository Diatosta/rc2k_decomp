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
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::rva;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::menus_store::MENUS;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{Callee, Carried, List, Screen, Step, log_line, screen_entry};

// ---- Rust version: `menu::screen_loop` with this ----

fn before(c: &mut Carried) -> Result<(), Refused> {
    c.measure_x = SCREEN_ITEMS.first[slot::VIEW_BEST_SCORES].get();
    SCREEN_ITEMS.current[slot::VIEW_BEST_SCORES].set(c.measure_x);
    Ok(())
}

unsafe fn set_up_first(_c: &mut Carried) -> Result<(), Refused> {
    // Between `pusha` and `popa`: no register changes.
    unsafe {
        log_line(c"BScoresScr: Start");
    }
    Ok(())
}

unsafe fn after(_c: &mut Carried) -> Result<(), Refused> {
    // Between `pusha` and `popa`: no register changes.
    unsafe {
        log_line(c"BScoresScr: End");
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.view_best_scores.addr(),
    saved_item: || &SCREEN_ITEMS.current[slot::VIEW_BEST_SCORES],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.view_best_scores_scene.addr(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.view_best_scores_draw.addr(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.setup_list, // dword_755500
            table: || DRAW_LISTS.view_best_scores_setup.addr(),
            run: true,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.view_best_scores_overlay.addr(),
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
        Step::ArgAddr(|| {
            crate::menu::texts_store::MENU_TEXTS
                .view_best_scores_picture
                .addr()
        }),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
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

/// The Rust version where the typed tables run it (a refusal is the
/// caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::view_best_scores_loop", &SCREEN)
}
