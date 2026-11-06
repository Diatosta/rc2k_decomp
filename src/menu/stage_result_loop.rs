//! `menu::stage_result_loop` (0x48558D): the loop of a stage's result screen. Seen after a
//! single race's stage (owner, 2026-10-22: the standalone reached it there); earlier
//! called a time trial's, wrongly.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `results1.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is asked about;
//! its own 3D list goes to `g_sceneList` before the first frame only.
//! Screen: the one player's result after a stage, "RESULTADOS": the rally and
//! stage, then "piloto", "tempo" and "velocidade máxima" (owner's screenshot from a Rust-run
//! visit, 2026-10-11). Named `screen_48558d_loop` until then.
//! Callers: `sub_482382`, `sub_48AE39`.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, FrameRoutine, List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: || crate::menu::owner::menus().tables.menus.stage_result.id(),
    saved_item: || &crate::menu::owner::menus().screens.items.current[slot::STAGE_RESULT],
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
                    .draw_758e7e
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
                    .overlay_7592c2
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
                    .stage_result_scene
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
                    .draw_758e7e
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
                    .overlay_7592c2
                    .id()
            },
            run: false,
        },
    ]),
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[Step::Write(
        || &crate::menu::owner::menus().screens.event.event_done,
        0x0,
    )],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|| {
        &crate::menu::owner::menus()
            .tables
            .texts
            .stage_result_picture
    }))],
    after_first_frame: &[],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Write(
        || &crate::menu::owner::menus().screens.option_work.menu_sound,
        0x6F,
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
