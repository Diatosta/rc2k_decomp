//! `menu::view_best_times_loop` (0x48E494): the loop of the best times screen opened from the
//! menus.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `btimes.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "MELHORES TEMPOS", not yet a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Entry, List, Screen, Step, screen_entry};

fn before(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.items.current[slot::VIEW_BEST_TIMES]
        .set(screens.items.first[slot::VIEW_BEST_TIMES].get());
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.view_best_times_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.view_best_times_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list, // dword_755500
            table: |m| m.tables.draw_lists.view_best_times_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.view_best_times_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Do(before)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.view_best_times_picture
    }))],
    ..Screen::new(
        |m| m.tables.menus.view_best_times.id(),
        |m| &m.screens.items.current[slot::VIEW_BEST_TIMES],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::view_best_times_loop", &SCREEN)
}
