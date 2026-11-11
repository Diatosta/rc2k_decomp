//! `menu::rally_stages_loop` (0x472596): the loop of the screen that shows a rally's stages,
//! on the map (`ukmap.png`), before the service area in a championship.
//!
//! The owner: "the screen before the service area in championship that shows that rally's stages".
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`): before the loop it clears the Escape flag `draw_work.escape_pressed`;
//! its background comes from `menu::load_background`, after which a set carry flag is fatal
//! error 0x80712345; leaving is asked about, and a "no" sets the screen up again.
//! Callers: the event loops `menu::run_championship`.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, Screen, Step};

// ---- game data ----

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.rally_stages_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.rally_stages_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.rally_stages_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Write(|m| &m.screens.draw_work.escape_pressed, 0)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.rally_stages_picture
    }))],
    asking: Some(Asking { ..Asking::DEFAULT }),
    // saved_item is the saved highlight
    ..Screen::new(
        |m| m.tables.menus.rally_stages.id(),
        |m| &m.screens.items.current[slot::RALLY_STAGES],
    )
};
