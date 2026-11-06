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
use crate::menu::owner::menus;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, FrameRoutine, List, Screen, Step, carousel, prepare_scene, quitting,
};
use crate::scene::draw_store::DRAW;

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let menus = menus();
    let screens = &menus.screens;
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0xEu32);
    carousel(entry.cars)?;
    let item = screens.items.current[slot::CAR_SPECS_47AF4C].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    screens.state.carousel.front.set(0x0u32);
    screens.event.shown_car.set(0x0u32);
    crate::menu::car_choice::find_player_car(menus)?;
    DRAW.view_offset[0].set(0x7u32);
    DRAW.view_offset[1].set(0xFFFFFFCEu32);
    DRAW.view_zoom.set(0x100u32);
    prepare_scene();
    Ok(())
}

fn after_first_frame(_: &Entry) -> Result<Flow, Refused> {
    if quitting() as u32 == 0x1 {
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    menu: || menus().tables.menus.car_specs_47af4c.id(),
    saved_item: || &menus().screens.items.current[slot::CAR_SPECS_47AF4C],
    lists: &[
        List {
            pointer: || &menus().screens.flow.scene_list, // g_sceneList
            table: || menus().tables.draw_lists.car_specs_47af4c_scene.id(),
            run: false,
        },
        List {
            pointer: || &menus().screens.flow.draw_list, // g_drawList
            table: || menus().tables.draw_lists.car_specs_47af4c_draw.id(),
            run: false,
        },
        List {
            pointer: || &menus().screens.flow.setup_list, // dword_755500
            table: || menus().tables.draw_lists.car_specs_47af4c_setup.id(),
            run: true,
        },
        List {
            pointer: || &menus().screens.flow.overlay_list, // g_overlayList
            table: || menus().tables.draw_lists.car_specs_47af4c_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|| {
        &menus().tables.texts.car_specs_47af4c_picture
    }))],
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
