//! `menu::time_trial_best_times_loop` (0x48CE48): the best times screen of the time trial, by
//! its title "MELHORES TEMPOS" and its callers (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `btimes.png`; leaving is asked about; left
//! without the question, the highlight is not saved; its own 3D list goes to `g_sceneList`
//! before the first frame only.
//! The name is a reading from the title string of its menu, "MELHORES TEMPOS", not yet a visit.
//! Callers: `sub_48AE39`, `sub_48AFD0`, `sub_48B23D`, `sub_48B54B`, `sub_48B90F`.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, FrameRoutine, List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: || {
        crate::menu::owner::menus()
            .tables
            .menus
            .time_trial_best_times
            .id()
    },
    saved_item: || &crate::menu::owner::menus().screens.items.current[slot::TIME_TRIAL_BEST_TIMES],
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
                    .draw_758d2a
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
                    .overlay_75921e
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
                    .time_trial_best_times_scene
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
                    .draw_758d2a
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
                    .overlay_75921e
                    .id()
            },
            run: false,
        },
    ]),
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|| {
        &crate::menu::owner::menus()
            .tables
            .texts
            .time_trial_best_times_picture
    }))],
    after_first_frame: &[],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: true,
    after: &[],
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
