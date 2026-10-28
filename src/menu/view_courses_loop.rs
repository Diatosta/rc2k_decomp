//! `menu::view_courses_loop` (0x48EC30): the loop of the course information screen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is not asked about; the steps that are more than
//! a call or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "INFORMAÇÃO DO PERCURSO", not yet a
//! visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::rva;
use crate::menu::arcade_store::ARCADE_GAME;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::menus_store::MENUS;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{Callee, Carried, List, Screen, Step, log_line, screen_entry};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(_c: &mut Carried) -> Result<(), Refused> {
    // Between `pusha` and `popa`: no register changes.
    unsafe {
        log_line(c"VCoursesScr: Start");
    }
    Ok(())
}

unsafe fn after(_c: &mut Carried) -> Result<(), Refused> {
    // Between `pusha` and `popa`: no register changes.
    unsafe {
        log_line(c"VCoursesScr: End");
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.view_courses.id(),
    saved_item: || &SCREEN_ITEMS.current[slot::VIEW_COURSES],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.view_courses_scene.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.view_courses_draw.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.setup_list, // dword_755500
            table: || DRAW_LISTS.view_courses_setup.id(),
            run: true,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.view_courses_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[
        Step::Write(|| &CHAMP_VIEW.stage_mode, 0x1),
        Step::Write(|| &EVENT_STATE.stage_group, 0x0),
        Step::Write(|| &EVENT_STATE.stage_number, 0x1),
        Step::Write(|| &EVENT_STATE.stage_sub, 0x0),
        Step::Write(|| &ARCADE_GAME.level, 0x0),
        Step::Write(|| &ARCADE_GAME.stage, 0x1),
        Step::Write(|| &EVENT_STATE.section, 0x0),
        Step::Write(|| &EVENT_STATE.index, 0x0),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::DrawStageMap)],
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
    screen_entry("menu::view_courses_loop", &SCREEN)
}
