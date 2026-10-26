//! `menu::best_class_scores_loop` (0x4A886B): the loop of the best championship scores screen,
//! by class.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `bascores.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is asked about;
//! its own 3D list goes to `g_sceneList` before the first frame only.
//! A reading from the title string of its menu, not yet a visit: "MELHORES CLASSIFICAÇÕES NO
//! CAMPEONATO".
//! Callers: `sub_4A8608`.

use crate::framework::rva;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::menus_store::MENUS;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{Callee, List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: || MENUS.best_class_scores.addr() as usize,
    saved_item: || &SCREEN_ITEMS.current[slot::BEST_CLASS_SCORES],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.unread_list, // g_unreadList
            table: || DRAW_LISTS.unread_758b72.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.draw_758d3a.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.overlay_759226.addr() as usize,
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.best_class_scores_scene.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.draw_758d3a.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.overlay_759226.addr() as usize,
            run: false,
        },
    ]),
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgAddr(|| {
            (crate::menu::texts_store::MENU_TEXTS
                .best_class_scores_picture
                .addr()) as usize
        }),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
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
