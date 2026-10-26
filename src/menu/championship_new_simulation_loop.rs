//! `menu::championship_new_simulation_loop` (0x4764DC): the start screen of the normal
//! championship, by its title "NOVA SIMULAÇÃO" and its place in the mode menu (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `sral.png`; leaving is not asked about; the
//! next frame begins with the steps after the first frame; the steps that are more than a call
//! or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "NOVA SIMULAÇÃO", not yet a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::rva;
use crate::menu::captions_store::CAPTIONS;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::menus_store::MENUS;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{
    Callee, Carried, Flow, List, Screen, Step, carousel, prepare_scene, screen_entry,
};
use crate::menu::state_store::MENU_STATE;
use crate::scene::draw_store::DRAW;

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        CAPTIONS.screen_left.set(0x0u32);
        SCREEN_FLOW.turning.set(0x1u32);
        MENU_STATE.carousel.menu.set(0x2u32);
        carousel(c)?;
        c.arg = SCREEN_ITEMS.current[slot::CHAMPIONSHIP_NEW_SIMULATION].get();
        LOOP_FLAGS.item.set(c.arg);
        c.arg = c.arg.wrapping_sub(1);
        MENU_STATE.carousel.front.set(c.arg);
        DRAW.view_offset[0].set(0x7u32);
        DRAW.view_offset[1].set(0xAu32);
        DRAW.view_zoom.set(0x100u32);
        prepare_scene(c);
    }
    Ok(())
}

fn after_first_frame(_c: &mut Carried) -> Result<Flow, Refused> {
    if CAPTIONS.screen_left.get() == 0x1 {
        CAPTIONS.screen_left.set(0x0u32);
        return Ok(Flow::SetUp);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.championship_new_simulation.addr(),
    saved_item: || &SCREEN_ITEMS.current[slot::CHAMPIONSHIP_NEW_SIMULATION],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.championship_new_simulation_scene.addr(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.championship_new_simulation_draw.addr(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.championship_new_simulation_overlay.addr(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Write(|| &EVENT_STATE.event_done, 0x0)],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgAddr(|| {
            crate::menu::texts_store::MENU_TEXTS
                .championship_new_simulation_picture
                .addr()
        }),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
    ],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: true,
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
    screen_entry("menu::championship_new_simulation_loop", &SCREEN)
}
