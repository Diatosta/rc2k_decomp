//! `menu::new_simulation_loop` (0x4A99AA): the loop of the screen that starts a new simulation.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `sral.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "NOVA SIMULAÇÃO", not yet a visit.
//! Callers: `sub_4A96BF`. The menus and draw lists run its `entry` (`menu::table_calls`); the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::menus_store::MENUS;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{
    Callee, Entry, FrameRoutine, List, Screen, Step, carousel, screen_entry,
};
use crate::menu::state_store::MENU_STATE;
use crate::scene::draw_store::DRAW;

// ---- Rust version: `menu::screen_loop` with this ----

fn before(_: &Entry) -> Result<(), Refused> {
    SCREEN_ITEMS.current[slot::NEW_SIMULATION].set(SCREEN_ITEMS.first[slot::NEW_SIMULATION].get());
    EVENT_STATE.event_done.set(0x0u32);
    Ok(())
}

fn set_up_first(_: &Entry) -> Result<(), Refused> {
    SCREEN_FLOW.turning.set(0x1u32);
    MENU_STATE.carousel.menu.set(0x2u32);
    carousel(None)?;
    let item = SCREEN_ITEMS.current[slot::NEW_SIMULATION].get();
    LOOP_FLAGS.item.set(item);
    MENU_STATE.carousel.front.set(item.wrapping_sub(1));

    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.new_simulation.id(),
    saved_item: || &SCREEN_ITEMS.current[slot::NEW_SIMULATION],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.new_simulation_scene.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.new_simulation_draw.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.new_simulation_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|| {
        &crate::menu::texts_store::MENU_TEXTS.new_simulation_picture
    }))],
    after_first_frame: &[
        Step::Write(|| &DRAW.view_offset[0], 0x7),
        Step::Write(|| &DRAW.view_offset[1], 0xA),
        Step::Write(|| &DRAW.view_zoom, 0x100),
        Step::Call(Callee::PrepareScene),
    ],
    confirm: None,
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

/// The Rust version where the typed tables run it (a refusal is the
/// caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::new_simulation_loop", &SCREEN)
}
