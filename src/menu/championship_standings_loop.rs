//! `menu::championship_standings_loop` (0x4670DF): the loop of the championship table, the
//! points of every driver in each of the six rallies and in total.
//!
//! In those screenshots the 3D objects of that menu (green arrows, a floppy disk) are drawn over
//! the table. This loop puts its own empty 3D list in `g_sceneList` only before the first frame
//! after set-up (and in the unread `flow.unread_list` at the other two places); the list of the menu before is cleared
//! with `Screen::first_frame_lists`, so the table shows without them.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`):
//! nothing happens before the loop (its set-up starts at the function's second byte); its
//! background comes from `menu::load_background`, after which a set carry flag is fatal error
//! 0x80712345; leaving is asked about, and a "no" sets the screen up again. If `state.menus.standings_shown` is set
//! when the screen is left, nothing more is done: no question, and the highlight is not saved. One
//! caller.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, Frames, List, Screen, Step};

// ---- game data ----

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list,
            table: |m| m.tables.draw_lists.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.championship_standings_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.championship_standings_overlay.id(),
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.championship_standings_picture
    }))],
    leave_at_once: Some(|m| &m.screens.state.menus.standings_shown),
    asking: Some(Asking { ..Asking::DEFAULT }),
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list, // g_sceneList
                table: |m| m.tables.draw_lists.championship_standings_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |m| m.tables.draw_lists.championship_standings_list.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |m| m.tables.draw_lists.championship_standings_overlay.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    // saved_item is the saved highlight
    ..Screen::new(
        |m| m.tables.menus.championship_standings.id(),
        |m| &m.screens.items.current[slot::CHAMPIONSHIP_STANDINGS],
    )
};
