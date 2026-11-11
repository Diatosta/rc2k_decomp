//! `menu::arcade_level_end_results_loop` (0x488F4C): the loop of a screen with the results background.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `results1.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is not asked
//! about.
//! Which screen it is has not been confirmed, hence the name. Its title string: "(empty)".
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, List, MenuSound, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.arcade_level_end_results_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.arcade_level_end_results_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.arcade_level_end_results_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Write(|m| &m.screens.event.event_over, 0x0)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.arcade_level_end_results_picture
    }))],
    after: &[Step::Sound(MenuSound::None)],
    ..Screen::new(
        |m| m.tables.menus.arcade_level_end_results.id(),
        |m| &m.screens.items.current[slot::ARCADE_LEVEL_END_RESULTS],
    )
};
