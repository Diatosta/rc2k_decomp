//! `menu::unused_cspecs_loop` (0x4AA1A3): a car specification screen with the carousel of
//! menu 0xC, background `cspecs.png`. Nothing in RAL.EXE reaches it; reached here from the
//! debug UI's "Screens" tab (`debug::screens`). What it was for is not known; a reading.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::menus_store::MENUS;
use crate::menu::screen_items_store::SCREEN_ITEMS;
use crate::menu::screen_loop::{
    Callee, Entry, FrameRoutine, List, Screen, Step, carousel, prepare_scene,
};
use crate::menu::state_store::MENU_STATE;
use crate::scene::draw_store::DRAW;

/// `unk_7476E4`'s slot, the item kept between visits.
const SLOT: usize = (0x7476E4 - 0x747660) / 4;

// ---- Rust version: `menu::screen_loop` with this ----

fn before(_: &Entry) -> Result<(), Refused> {
    SCREEN_ITEMS.current[SLOT].set(SCREEN_ITEMS.first[SLOT].get());
    Ok(())
}

// The original prepares the scene before the carousel, which left the front car white
// (seen 2026-10-30): here as `menu::select_car_loop`, the scene last.
fn set_up_first(_: &Entry) -> Result<(), Refused> {
    SCREEN_FLOW.turning.set(0x1u32);
    MENU_STATE.carousel.menu.set(0xCu32);
    carousel(None)?;
    LOOP_FLAGS.item.set(SCREEN_ITEMS.current[SLOT].get());
    SCREEN_FLOW.turning.set(0x0u32);
    MENU_STATE.carousel.front.set(0x0u32);
    EVENT_STATE.shown_car.set(0x0u32);
    DRAW.view_offset[0].set(0x7u32);
    DRAW.view_offset[1].set(0xFFFFFFCEu32);
    DRAW.view_zoom.set(0x100u32);
    prepare_scene();
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.menu_74ba3c.id(),
    saved_item: || &SCREEN_ITEMS.current[SLOT],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.list_758f6e.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.list_758c52.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.setup_list, // dword_755500
            table: || DRAW_LISTS.list_75941a.id(),
            run: true,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.list_7591b2.id(),
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
        &crate::menu::texts_store::MENU_TEXTS.cspecs_png
    }))],
    after_first_frame: &[],
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
