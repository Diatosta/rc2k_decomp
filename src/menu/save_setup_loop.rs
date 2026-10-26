//! `menu::save_setup_loop` (0x46EB7A): the loop of the screen where a car setup is saved and
//! its name typed.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is not asked about; it has a step of its own once
//! a frame; the steps that are more than a call or a store are the functions below, generated
//! from the listing.
//! The name is a reading from the title string of its menu, "{}", not yet a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::rva;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::menus_store::MENUS;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{
    Callee, Carried, Flow, List, Screen, Step, prepare_scene, screen_entry,
};
use crate::scene::draw_store::DRAW;

// ---- Rust version: `menu::screen_loop` with this ----

fn prepare(c: &mut Carried) -> Result<(), Refused> {
    DRAW.view_offset[0].set(0x0u32);
    DRAW.view_offset[1].set(0x0u32);
    DRAW.view_zoom.set(0x100u32);
    prepare_scene(c);
    DRAW.clip_edges[2].set(0x0u32);
    DRAW.clip_edges[3].set(0x320u32);
    DRAW.clip_screen[2].set(DRAW.clip_edges[2].bits() as i32 as f32);
    DRAW.clip_screen[3].set(DRAW.clip_edges[3].bits() as i32 as f32);
    Ok(())
}

unsafe fn each_frame(c: &mut Carried) -> Result<Flow, Refused> {
    unsafe {
        if OPTION_WORK.typing.get() == 0x1 {
            Callee::EditSetupName.run(c)?;
            if OPTION_WORK.typing.get() != 0x0 {
                return Ok(Flow::NextFrame);
            }
            DRAW_WORK.action_value.set(0x2u32);
            return Ok(Flow::Leave);
        }
        Ok(Flow::On)
    }
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.save_setup.addr() as usize,
    saved_item: || &SCREEN_ITEMS.current[slot::SAVE_SETUP],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.save_setup_scene.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.save_setup_draw.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.save_setup_overlay.addr() as usize,
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Write(|| &SCREEN_FLOW.left_service, 0x0)],
    prepare: &[Step::Do(prepare)],
    after_lists: &[],
    background: &[],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Write(|| &SCREEN_FLOW.left_service, 0x1)],
    escape_leaves: false,
    each_frame: &[Step::DoFlow(each_frame)],
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

/// The Rust version where the typed tables run it (a refusal is the
/// caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::save_setup_loop", &SCREEN)
}
