//! The loop of a screen with the results background. Screen-specific part of `menu::screen_loop`:
//! background `results1.png` through `menu::load_background`; leaving is not asked about.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, List, MenuSound, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.arcade_level_end_results_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.arcade_level_end_results_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.arcade_level_end_results_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(|m| &m.screens.event.event_over, false)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.arcade_level_end_results_picture
    }))],
    after: &[Step::Sound(MenuSound::None)],
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.arcade_level_end_results.id(),
        |m| &m.screens.items.current[slot::ARCADE_LEVEL_END_RESULTS],
    )
};
