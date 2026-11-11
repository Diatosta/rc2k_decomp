//! `menu::class_championship_standings_loop` (0x4A4B6B): the loop of the championship table for
//! the player's class.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `champoc.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is asked about;
//! returns at once on leaving if `dword_75AB34` is 1.
//! A reading from the title string of its menu, not yet a visit: "CAMPEONATO DE CLASSE".
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.class_championship_standings_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.class_championship_standings_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| {
                m.tables
                    .draw_lists
                    .class_championship_standings_overlay
                    .id()
            },
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.class_championship_standings_picture
    }))],
    leave_at_once: Some(|m| &m.screens.state.menus.standings_shown),
    asking: Some(Asking { ..Asking::DEFAULT }),
    ..Screen::new(
        |m| m.tables.menus.class_championship_standings.id(),
        |m| &m.screens.items.current[slot::CLASS_CHAMPIONSHIP_STANDINGS],
    )
};
