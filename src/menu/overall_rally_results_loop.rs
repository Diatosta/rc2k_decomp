//! `menu::overall_rally_results_loop` (0x466B21): the loop of the fourth results screen after a
//! stage, the rally's standings so far for all classes.
//!
//! Named `screen_466b21_loop` until then.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`): before the loop it sets the string of the menu's first item to 0x138 if
//! `dword_75AADC` is 1 and to 0x139 otherwise, as the stage results screen does;
//! its background comes from `menu::load_background`, after which a set carry flag is fatal
//! error 0x80712345; leaving is asked about, and a "no" sets the screen up again.
//! If `dword_75AB34` is 1 when the screen is left, nothing more is done: no question,
//! and the highlight is not saved.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, Entry, List, Screen, Step};

// ---- game data ----

fn set_first_item(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    let title = if menus.screens.state.menus.which_title.get() == 1 {
        0x138
    } else {
        0x139
    };
    menus.tables.menus.overall_rally_results.id().items()[0]
        .string
        .set(title);

    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.overall_rally_results_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.list_758e36.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.overall_rally_results_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Do(set_first_item)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.overall_rally_results_picture
    }))],
    leave_at_once: Some(|m| &m.screens.state.menus.standings_shown), // dword_75AB34
    asking: Some(Asking { ..Asking::DEFAULT }),
    // saved_item is dword_74778C
    ..Screen::new(
        |m| m.tables.menus.overall_rally_results.id(),
        |m| &m.screens.items.current[slot::OVERALL_RALLY_RESULTS],
    )
};
