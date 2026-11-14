//! The loop of the championship table for the player's class. Screen-specific part of
//! `menu::screen_loop`: background `champoc.png` through `menu::load_background`; leaving is asked
//! about; returns at once on leaving if `state.menus.standings_shown` is set.
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, Screen, Step};
use crate::menu::texts_store::texts;

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.class_championship_standings_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.class_championship_standings_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.class_championship_standings_overlay.id(),
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().class_championship_standings_picture
    }))],
    leave_at_once: Some(|m| &m.screens.state.menus.standings_shown),
    asking: Some(Asking { ..Asking::DEFAULT }),
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.class_championship_standings.id(),
        |m| &m.screens.items.current[slot::CLASS_CHAMPIONSHIP_STANDINGS],
    )
};
