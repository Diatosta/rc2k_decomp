//! `menu::select_stage_loop` (0x482099): the stage selection "ESCOLHA ETAPA" of the start menus
//! under sub_482382.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; left without the question, the
//! highlight is not saved; the next frame begins with the steps after the first frame; the
//! question is not put while `dword_52A910` is 1; the steps that are more than a call or a
//! store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "ESCOLHA ETAPA", not yet a visit.
//! Callers: `sub_418347`, `sub_482382`, `sub_4824C1`, `sub_4826A2`, `sub_4828B0`, `sub_482B7A`.
//! Rust callers run its `entry` (`routes.rs`); the registers above are the original's, for
//! reference.

use crate::errors::Refused;
use crate::menu::captions_store::CAPTIONS;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::menus_store::MENUS;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::pages_store::MENU_PAGES;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{Callee, Entry, Flow, FrameRoutine, List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

fn after_first_frame(_: &Entry) -> Result<Flow, Refused> {
    if CAPTIONS.screen_left.get() == 0x1 {
        CAPTIONS.screen_left.set(0x0u32);
        return Ok(Flow::SetUp);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.select_stage.id(),
    saved_item: || &SCREEN_ITEMS.current[slot::SELECT_STAGE],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.select_stage_scene.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.select_stage_draw.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.setup_list, // dword_755500
            table: || DRAW_LISTS.select_stage_setup.id(),
            run: true,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.select_stage_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[Step::Write(|| &MENU_PAGES.yes_no_answer, 0x0)],
    set_up_first: &[Step::Call(Callee::MarkStageChoice)],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::DrawStageMap)],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: true,
    after: &[Step::Write(|| &OPTION_WORK.menu_sound, 0x6F)],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: true,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: false,
    before_asking: &[],
    ask_unless: Some(|| crate::net::store::NET.session.get()),
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
