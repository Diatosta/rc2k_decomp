//! `menu::inspection_area_loop` (0x47DBF3): the loop of the inspection area screen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`), as
//! extracted from the disassembly: background `service.png` through `menu::load_background` (carry
//! afterwards = fatal error 0x80712345); leaving is asked about. A reading from the title string of
//! its menu, not yet a visit: "ÁREA DE INSPECÇÃO".
//! Callers: `sub_48E093`, `sub_4A15E3`, `4A1EC7`, `sub_4A8556`.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, FrameRoutine, List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: || {
        crate::menu::owner::menus()
            .tables
            .menus
            .inspection_area
            .id()
    },
    saved_item: || &crate::menu::owner::menus().screens.items.current[slot::INSPECTION_AREA],
    lists: &[
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.scene_list, // g_sceneList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .inspection_area_scene
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
                    .inspection_area_draw
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
                    .inspection_area_overlay
                    .id()
            },
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: Some(FrameRoutine::Inspection),
    before: &[],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::Call(Callee::LoadPicture(|| {
            &crate::menu::owner::menus()
                .tables
                .texts
                .inspection_area_picture
        })),
        Step::CallOrFatal(Callee::ConvertPicture, 0x80712345),
    ],
    after_first_frame: &[],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
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
