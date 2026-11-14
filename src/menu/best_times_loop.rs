//! The loop of the best times screen. Screen-specific part of `menu::screen_loop`: background
//! `btimes.png` through `menu::load_background` (a failed load is fatal error 0x80712345);
//! leaving is asked about.
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, Screen, Step};
use crate::menu::texts_store::texts;

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.best_times_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.best_times_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.best_times_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.best_times_overlay.id(),
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().best_times_picture
    }))],
    asking: Some(Asking { ..Asking::DEFAULT }),
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.best_times.id(),
        slot::BEST_TIMES,
    )
};
