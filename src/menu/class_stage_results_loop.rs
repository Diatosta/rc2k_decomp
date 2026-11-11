//! `menu::class_stage_results_loop` (0x4662C6): the loop of the first results screen after a
//! stage, the stage's times for the player's class.
//!
//! Named `screen_4662c6_loop`, then `stage_results_loop`.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`):
//! before the loop it sets the string of the menu's first item, the button at the bottom right, to
//! 0x138 ("voltar", back) if `state.menus.which_title` is 1 and to 0x139 ("continuar") otherwise, as all four
//! results screens do; its background comes from `menu::load_background` (0x47EFFF), after which a
//! set carry flag is fatal error 0x80712345 (the flag is whatever the last blit in there left);
//! leaving is asked about, and a "no" sets the screen up again. One caller.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, Entry, List, Screen, Step};

// ---- game data ----

fn set_first_item(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    let title = if menus.screens.state.menus.which_title.is_set() {
        0x138
    } else {
        0x139
    };
    menus.tables.menus.class_stage_results.id().items()[0]
        .string
        .set(title);

    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.class_stage_results_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.class_stage_results_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.class_stage_results_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Do(set_first_item)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.class_stage_results_picture
    }))],
    asking: Some(Asking { ..Asking::DEFAULT }),
    // saved_item is the saved highlight
    ..Screen::new(
        |m| m.tables.menus.class_stage_results.id(),
        |m| &m.screens.items.current[slot::CLASS_STAGE_RESULTS],
    )
};
