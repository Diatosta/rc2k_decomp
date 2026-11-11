//! `menu::leave_question_loop` (0x47FA94): the loop of the "leave?" question that
//! `menu::confirm_leaving` puts.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `mainscr.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is not asked
//! about; its own 3D list goes to `g_sceneList` before the first frame only.
//! A reading, not yet a visit: its one caller is `menu::confirm_leaving`, and it sets
//! `g_askingYesNo` to 1 while it is up, which is what makes `handle_yes_no` listen.
//! Callers: `menu::confirm_leaving`. Rust callers run its `entry` (`routes.rs`).
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Frames, List, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list, // g_unreadList
            table: |m| m.tables.draw_lists.unread_758b72.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.draw_758c9e.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.overlay_7591de.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Write(|m| &m.screens.flow.asking, 0x1)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.leave_question_picture
    }))],
    after: &[Step::Write(|m| &m.screens.flow.asking, 0x0)],
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list, // g_sceneList
                table: |m| m.tables.draw_lists.leave_question_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list, // g_drawList
                table: |m| m.tables.draw_lists.draw_758c9e.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
                table: |m| m.tables.draw_lists.overlay_7591de.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.leave_question.id(),
        |m| &m.screens.items.current[slot::LEAVE_QUESTION],
    )
};
