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
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, FrameRoutine, List, Screen, Step, log_line, screen_entry,
};

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(_: &Entry) -> Result<(), Refused> {
    // Between `pusha` and `popa`: no register changes.
    log_line(c"VCoursesScr: Start");
    Ok(())
}

fn after(_: &Entry) -> Result<(), Refused> {
    // Between `pusha` and `popa`: no register changes.
    log_line(c"VCoursesScr: End");
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: |m| m.tables.menus.view_courses.id(),
    saved_item: |m| &m.screens.items.current[slot::VIEW_COURSES],
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.view_courses_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.view_courses_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list, // dword_755500
            table: |m| m.tables.draw_lists.view_courses_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.view_courses_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[
        Step::Write(|m| &m.play.champ_view.stage_mode, 0x1),
        Step::Write(|m| &m.screens.event.stage_group, 0x0),
        Step::Write(|m| &m.screens.event.stage_number, 0x1),
        Step::Write(|m| &m.screens.event.stage_sub, 0x0),
        Step::Write(|m| &m.play.arcade.level, 0x0),
        Step::Write(|m| &m.play.arcade.stage, 0x1),
        Step::Write(|m| &m.screens.event.section, 0x0),
        Step::Write(|m| &m.screens.event.index, 0x0),
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
