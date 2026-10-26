//! `menu::arcade_next_stage_loop` (0x48980C): a screen titled "ETAPA SEGUINTE" under the mode
//! menu's third entry (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; left without the question, the
//! highlight is not saved.
//! The name is a reading from the title string of its menu, "ETAPA SEGUINTE", not yet a visit.
//! Callers: `sub_487E2C`.

use crate::framework::rva;
use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::menus_store::MENUS;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::pages_store::MENU_PAGES;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{Callee, List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: || MENUS.arcade_next_stage.addr() as usize,
    saved_item: || &SCREEN_ITEMS.current[slot::ARCADE_NEXT_STAGE],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.arcade_next_stage_scene.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.arcade_next_stage_draw.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.arcade_next_stage_overlay.addr() as usize,
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Write(|| &MENU_PAGES.yes_no_answer, 0x0)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[Step::Write(|| &CHAMP_VIEW.stage_mode, 0x2)],
    background: &[Step::Call(Callee::DrawStageMap)],
    after_first_frame: &[],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: true,
    after: &[Step::Write(|| &OPTION_WORK.menu_sound, 0x6F)],
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
