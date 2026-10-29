//! `menu::rally_stages_loop` (0x472596): the loop of the screen that shows a rally's stages,
//! on the map (`ukmap.png`), before the service area in a championship.
//!
//! The owner: "the screen before the service area in championship that shows that rally's stages".
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`): before the loop it clears the Escape flag `dword_755A3C`;
//! its background comes from `menu::load_background`, after which a set carry flag is fatal
//! error 0x80712345; leaving is asked about, and a "no" sets the screen up again.
//! Callers: the event loops `sub_4A0E60` and `sub_4AA4D7`.

use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::menus_store::MENUS;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{Callee, FRAME_CALLBACK, List, Screen, Step};

// ---- game data ----

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: || MENUS.rally_stages.id(),
    saved_item: || &SCREEN_ITEMS.current[slot::RALLY_STAGES], // dword_747770
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list,
            table: || DRAW_LISTS.rally_stages_scene.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list,
            table: || DRAW_LISTS.list_758d56.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list,
            table: || DRAW_LISTS.rally_stages_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FRAME_CALLBACK,
    later_frame_callback: None,
    before: &[Step::Write(|| &DRAW_WORK.escaped, 0)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgText(|| &crate::menu::texts_store::MENU_TEXTS.rally_stages_picture),
        Step::CallOrFatal(Callee::LoadBackground, 0x8071_2345),
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
