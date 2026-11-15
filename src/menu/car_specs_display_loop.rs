//! A screen over the car specification background, called from `menu::show_car_specs` and
//! `menu::arcade_level_end`. Screen-specific part of `menu::screen_loop`: background `cspecs.png`;
//! leaving is not asked about.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, List, Screen, Step, carousel, prepare_scene, quitting,
};
use crate::menu::texts_store::texts;

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    let screens = &menus.screens;
    screens.flow.turning.set(0x1u32);
    screens.carousel.menu.set(0xEu32);
    carousel(entry, entry.cars)?;
    let item = screens.items.current[slot::CAR_SPECS_DISPLAY].get();
    screens.loop_flags.item.set(item);
    screens.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    screens.carousel.front.set(0x0u32);
    screens.event.shown.car.set(0x0u32);
    crate::menu::car_choice::find_player_car(menus)?;
    entry.set_view(7, -50, 0x100);
    prepare_scene(entry);
    Ok(())
}

fn after_first_frame(entry: &Entry) -> Result<Flow, Refused> {
    if quitting(entry.system) as u32 == 0x1 {
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.car_specs_display_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.car_specs_display_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.car_specs_display_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.car_specs_display_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().car_specs_display_picture
    }))],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.car_specs_display.id(),
        slot::CAR_SPECS_DISPLAY,
    )
};
