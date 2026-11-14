//! `menu::championship_standings_loop`: the loop of the championship table, the points of every
//! driver in each of the six rallies and in total. The 3D objects of the menu before (green arrows,
//! a floppy disk) would be drawn over the table. This loop puts its own empty 3D list in the scene
//! list only before the first frame after set-up (and in the unread `flow.unread_list` at the other
//! two places); the list of the menu before is cleared with `Screen::first_frame_lists`, so the
//! table shows without them. Screen-specific part of `menu::screen_loop`: nothing happens before
//! the loop; its background comes from `menu::load_background` (a failed load is fatal
//! error 0x80712345); leaving is asked about, and a "no" sets the screen up again. If
//! `state.menus.standings_shown` is set when the screen is left, nothing more is done: no question,
//! and the highlight is not saved. One caller.
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, Frames, List, Screen, Step};
use crate::menu::texts_store::texts;

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list,
            table: |_| DRAW_LISTS.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.championship_standings_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.championship_standings_overlay.id(),
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().championship_standings_picture
    }))],
    leave_at_once: Some(|m| &m.screens.state.menus.standings_shown),
    asking: Some(Asking { ..Asking::DEFAULT }),
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |_| DRAW_LISTS.championship_standings_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.championship_standings_list.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.championship_standings_overlay.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.championship_standings.id(),
        |m| &m.screens.items.current[slot::CHAMPIONSHIP_STANDINGS],
    )
};
