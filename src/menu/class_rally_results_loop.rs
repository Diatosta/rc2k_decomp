//! The loop of the third results screen after a stage, the rally's standings so far for the
//! player's class. Screen-specific part of `menu::screen_loop`: before the loop it sets the
//! string of the menu's first item to 0x138 if `state.menus.which_title` is 1 and to 0x139
//! otherwise, as the stage results screen does; its background comes from
//! `menu::load_background` (a failure is fatal error 0x80712345); leaving is asked about, and a
//! "no" sets the screen up again. One caller.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Menus;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, Screen, Step};

fn set_first_item(menus: &Menus) -> Result<(), Refused> {
    let title = if menus.screens.state.menus.which_title.is_set() {
        0x138
    } else {
        0x139
    };
    menus
        .tables
        .menus
        .class_rally_results
        .id()
        .items(&menus.tables.menus)[0]
        .string
        .set(title);

    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.class_rally_results_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.class_rally_results_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.class_rally_results_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::DoMenus(set_first_item)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.class_rally_results_picture
    }))],
    asking: Some(Asking { ..Asking::DEFAULT }),
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.class_rally_results.id(),
        slot::CLASS_RALLY_RESULTS,
    )
};
