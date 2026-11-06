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
use crate::menu::owner::menus;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Entry, Flow, FrameRoutine, List, Screen, Step};
use crate::net::owner::net;

// ---- Rust version: `menu::screen_loop` with this ----

fn after_first_frame(_: &Entry) -> Result<Flow, Refused> {
    let tables = &menus().tables;
    if tables.captions.screen_left.get() == 0x1 {
        tables.captions.screen_left.set(0x0u32);
        return Ok(Flow::SetUp);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    menu: || menus().tables.menus.select_stage.id(),
    saved_item: || &menus().screens.items.current[slot::SELECT_STAGE],
    lists: &[
        List {
            pointer: || &menus().screens.flow.scene_list, // g_sceneList
            table: || menus().tables.draw_lists.select_stage_scene.id(),
            run: false,
        },
        List {
            pointer: || &menus().screens.flow.draw_list, // g_drawList
            table: || menus().tables.draw_lists.select_stage_draw.id(),
            run: false,
        },
        List {
            pointer: || &menus().screens.flow.setup_list, // dword_755500
            table: || menus().tables.draw_lists.select_stage_setup.id(),
            run: true,
        },
        List {
            pointer: || &menus().screens.flow.overlay_list, // g_overlayList
            table: || menus().tables.draw_lists.select_stage_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[Step::Write(|| &menus().tables.pages.yes_no_answer, 0x0)],
    set_up_first: &[Step::Call(Callee::MarkStageChoice)],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::DrawStageMap)],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: true,
    after: &[Step::Write(
        || &menus().screens.option_work.menu_sound,
        0x6F,
    )],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: true,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: false,
    before_asking: &[],
    ask_unless: Some(|| net().vars.session.get()),
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
