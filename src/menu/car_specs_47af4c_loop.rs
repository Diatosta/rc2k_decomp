//! `menu::car_specs_47af4c_loop` (0x47AF4C): a screen over the car specification background,
//! called from sub_46FD31 and sub_48AD73.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `cspecs.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from its background and callers, not yet a visit.
//! Callers: `sub_46FD31`, `sub_46FD31`, `sub_46FD31`, `sub_46FD31`, `sub_46FD31`, `sub_48AD73`.

use crate::errors::Refused;
use crate::framework::rva;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::menus_store::MENUS;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{
    Callee, Carried, Flow, List, Screen, Step, carousel, prepare_scene, quitting,
};
use crate::menu::state_store::MENU_STATE;
use crate::scene::draw_store::DRAW;

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    SCREEN_FLOW.turning.set(0x1u32);
    MENU_STATE.carousel.menu.set(0xEu32);
    carousel(c)?;
    c.arg = SCREEN_ITEMS.current[slot::CAR_SPECS_47AF4C].get();
    LOOP_FLAGS.item.set(c.arg);
    c.arg = c.arg.wrapping_sub(1);
    MENU_STATE.carousel.front.set(c.arg);
    SCREEN_FLOW.turning.set(0x0u32);
    MENU_STATE.carousel.front.set(0x0u32);
    EVENT_STATE.shown_car.set(0x0u32);
    crate::menu::car_choice::find_player_car()?;
    DRAW.view_offset[0].set(0x7u32);
    DRAW.view_offset[1].set(0xFFFFFFCEu32);
    DRAW.view_zoom.set(0x100u32);
    prepare_scene(c);
    Ok(())
}

fn after_first_frame(_c: &mut Carried) -> Result<Flow, Refused> {
    if quitting() as u32 == 0x1 {
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.car_specs_47af4c.addr() as usize,
    saved_item: || &SCREEN_ITEMS.current[slot::CAR_SPECS_47AF4C],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.car_specs_47af4c_scene.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.car_specs_47af4c_draw.addr() as usize,
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.setup_list, // dword_755500
            table: || DRAW_LISTS.car_specs_47af4c_setup.addr() as usize,
            run: true,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.car_specs_47af4c_overlay.addr() as usize,
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgAddr(|| {
            (crate::menu::texts_store::MENU_TEXTS
                .car_specs_47af4c_picture
                .addr()) as usize
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
