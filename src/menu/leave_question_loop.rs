//! `menu::leave_question_loop` (0x47FA94): the loop of the "leave?" question that
//! `menu::confirm_leaving` puts.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `mainscr.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is not asked
//! about; its own 3D list goes to `g_sceneList` before the first frame only.
//! A reading, not yet a visit: its one caller is `menu::confirm_leaving`, and it sets
//! `g_askingYesNo` to 1 while it is up, which is what makes `handle_yes_no` listen.
//! Callers: `menu::confirm_leaving`. Rust callers run its `entry` (`routes.rs`); the registers
//! above are the original's, for reference.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, FrameRoutine, List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: || crate::menu::owner::menus().tables.menus.leave_question.id(),
    saved_item: || &crate::menu::owner::menus().screens.items.current[slot::LEAVE_QUESTION],
    lists: &[
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.unread_list, // g_unreadList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .unread_758b72
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.draw_list, // g_drawList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .draw_758c9e
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.overlay_list, // g_overlayList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .overlay_7591de
                    .id()
            },
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.scene_list, // g_sceneList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .leave_question_scene
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.draw_list, // g_drawList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .draw_758c9e
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.overlay_list, // g_overlayList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .overlay_7591de
                    .id()
            },
            run: false,
        },
    ]),
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Write(
        || &crate::menu::owner::menus().screens.flow.asking,
        0x1,
    )],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|| {
        &crate::menu::owner::menus()
            .tables
            .texts
            .leave_question_picture
    }))],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Write(
        || &crate::menu::owner::menus().screens.flow.asking,
        0x0,
    )],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: false,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: false,
    before_asking: &[],
    ask_unless: None,
    leaving: &[],
    leave_saved_item: None,
    keys_menu: None,
    after_restored: &[],
    result: None,
    late_highlight: false,
    before_first_frame: &[],
    before_frame: &[],
    after_blink: &[],
    restart_action_check: false,
    always_asks: false,
};
