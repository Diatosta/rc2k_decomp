//! `menu::best_times_loop` (0x4A3EF3): the loop of the best times screen.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `btimes.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is asked about.
//! A reading from the title string of its menu, not yet a visit: "MELHORES TEMPOS".
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.best_times_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.best_times_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list, // dword_755500
            table: |m| m.tables.draw_lists.best_times_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.best_times_overlay.id(),
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.best_times_picture
    }))],
    asking: Some(Asking { ..Asking::DEFAULT }),
    ..Screen::new(
        |m| m.tables.menus.best_times.id(),
        |m| &m.screens.items.current[slot::BEST_TIMES],
    )
};
