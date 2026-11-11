//! `menu::load_championship_table_loop` (0x46E899): a second screen titled "CARREGAR
//! CAMPEONATO", reached through a table.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is not asked about; on leaving the highlight is
//! saved in `dword_747678`, not in the variable it is taken from (the original's slip); the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "CARREGAR CAMPEONATO", not yet a
//! visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Entry, List, Screen, Step, prepare_scene, screen_entry};

fn prepare(entry: &Entry) -> Result<(), Refused> {
    entry.set_view(0, 0, 0x100);
    prepare_scene(entry.scene);
    entry.set_clip_sides(0, 800);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.load_championship_table_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.load_championship_table_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.load_championship_table_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Write(|m| &m.screens.flow.left_service, 0x0)],
    prepare: &[Step::Do(prepare)],
    after: &[Step::Write(|m| &m.screens.flow.left_service, 0x1)],
    leave_saved_item: Some(|m| &m.screens.items.current[slot::SAVE_SETUP]),
    ..Screen::new(
        |m| m.tables.menus.load_championship_table.id(),
        |m| &m.screens.items.current[slot::LOAD_CHAMPIONSHIP_TABLE],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::load_championship_table_loop", &SCREEN)
}
