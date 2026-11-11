//! `menu::car_specs_display_loop` (0x47AF4C): a screen over the car specification background,
//! called from `menu::show_car_specs` and `menu::arcade_level_end`.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `cspecs.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from its background and callers, not yet a visit.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, List, Screen, Step, carousel, prepare_scene, quitting,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    let screens = &menus.screens;
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0xEu32);
    carousel(entry.cars)?;
    let item = screens.items.current[slot::CAR_SPECS_DISPLAY].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    screens.state.carousel.front.set(0x0u32);
    screens.event.shown_car.set(0x0u32);
    crate::menu::car_choice::find_player_car(menus)?;
    crate::scene::polygon_work::front_end().window.view_offset[0].set(0x7u32);
    crate::scene::polygon_work::front_end().window.view_offset[1].set(0xFFFFFFCEu32);
    crate::scene::polygon_work::front_end()
        .window
        .view_zoom
        .set(0x100u32);
    prepare_scene(entry.scene);
    Ok(())
}

fn after_first_frame(_: &Entry) -> Result<Flow, Refused> {
    if quitting() as u32 == 0x1 {
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.car_specs_display_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.car_specs_display_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list, // dword_755500
            table: |m| m.tables.draw_lists.car_specs_display_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.car_specs_display_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.car_specs_display_picture
    }))],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    ..Screen::new(
        |m| m.tables.menus.car_specs_display.id(),
        |m| &m.screens.items.current[slot::CAR_SPECS_DISPLAY],
    )
};
