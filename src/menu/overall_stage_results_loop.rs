//! `menu::overall_stage_results_loop`: the loop of the second results screen after a stage, the
//! stage's times for all classes.
//!
//! Screen-specific part of `menu::screen_loop`: before the loop it sets the string of the menu's
//! first item to 0x138 if `state.menus.which_title` is 1 and to 0x139 otherwise, as the stage
//! results screen does; its background comes from `menu::load_background`, a failed load being
//! fatal error 0x80712345; leaving is asked about, and a "no" sets the screen up again. One caller.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Menus;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, Screen, Step};
use crate::menu::texts_store::texts;

fn set_first_item(menus: &Menus) -> Result<(), Refused> {
    let title = if menus.screens.state.menus.which_title.is_set() {
        0x138
    } else {
        0x139
    };
    menus
        .tables
        .menus
        .overall_stage_results
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
            table: |_| DRAW_LISTS.overall_stage_results_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.overall_stage_results_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.overall_stage_results_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::DoMenus(set_first_item)],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().overall_stage_results_picture
    }))],
    asking: Some(Asking { ..Asking::DEFAULT }),
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.overall_stage_results.id(),
        slot::OVERALL_STAGE_RESULTS,
    )
};
