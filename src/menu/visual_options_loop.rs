//! `menu::visual_options_loop` (0x480592): the loop of the visual options screen.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `visual.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is not asked
//! about.
//! A reading from the title string of its menu, not yet a visit: "OPÇÕES VISUAIS".
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, List, Screen, Step, screen_entry};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.visual_options_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.visual_options_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.visual_options_overlay.id(),
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.visual_options_picture
    }))],
    ..Screen::new(
        |m| m.tables.menus.visual_options.id(),
        |m| &m.screens.items.current[slot::VISUAL_OPTIONS],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::visual_options_loop", &SCREEN)
}
