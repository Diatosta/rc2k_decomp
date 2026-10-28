//! `menu::inspection_area_loop` (0x47DBF3): the loop of the inspection area screen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`), as
//! extracted from the disassembly: background `service.png` through `menu::load_background` (carry
//! afterwards = fatal error 0x80712345); leaving is asked about. A reading from the title string of
//! its menu, not yet a visit: "ÁREA DE INSPECÇÃO".
//! Callers: `sub_48E093`, `sub_4A15E3`, `4A1EC7`, `sub_4A8556`.

use crate::framework::rva;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::menus_store::MENUS;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{Callee, List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: || MENUS.inspection_area.id(),
    saved_item: || &SCREEN_ITEMS.current[slot::INSPECTION_AREA],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.inspection_area_scene.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.inspection_area_draw.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.inspection_area_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: Some(rva(0x46D019)), // sub_46D019
    before: &[],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgAddr(|| {
            (crate::menu::texts_store::MENU_TEXTS
                .inspection_area_picture
                .addr()) as usize
        }),
        Step::Call(Callee::LoadPicture),
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
