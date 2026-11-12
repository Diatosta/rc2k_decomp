//! `menu::view_courses_loop` (0x48EC30): the loop of the course information screen.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is not asked about; the steps that are more than
//! a call or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "INFORMAÇÃO DO PERCURSO", not yet a
//! visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Entry, List, Screen, Step, log_line, screen_entry};

fn set_up_first(_: &Entry) -> Result<(), Refused> {
    log_line(c"VCoursesScr: Start");
    Ok(())
}

fn after(_: &Entry) -> Result<(), Refused> {
    log_line(c"VCoursesScr: End");
    Ok(())
}

pub const SCREEN: Screen = Screen {
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
            pointer: |m| &m.screens.flow.setup_list,
            table: |m| m.tables.draw_lists.view_courses_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.view_courses_overlay.id(),
            run: false,
        },
    ],
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
    background: &[Step::Call(Callee::DrawStageMap)],
    after: &[Step::Do(after)],
    ..Screen::new(
        |m| m.tables.menus.view_courses.id(),
        |m| &m.screens.items.current[slot::VIEW_COURSES],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::view_courses_loop", &SCREEN)
}
