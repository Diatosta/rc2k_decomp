//! `menu::select_car_loop` (0x47AC0C): the car selection screen, which the starters of
//! every kind of event run (the car select menu).
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`): background `cspecs.png`; leaving is not asked about; it has steps of its own
//! first on leaving.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, List, Screen, Step, carousel, prepare_scene, quitting,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    let screens = &menus.screens;
    // The current player's +0x30 (its car).
    let current: u32 = screens.event.current_player.get();
    let Some(player) = crate::menu::player::player(menus, current) else {
        return Err(crate::errors::BadInput("a current player past the four").into());
    };
    screens.flow.shown_gearing.set(player.get().gearing);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0xDu32);
    carousel(entry, entry.cars)?;
    let item = screens.items.current[slot::SELECT_CAR].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    screens.state.carousel.front.set(0x0u32);
    screens.event.shown_car.set(0x0u32);
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

fn leaving(entry: &Entry) -> Result<(), Refused> {
    if entry.menus.tables.captions.left_by_key.is_set() {
        Callee::GoBack.run(entry)?;
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.select_car_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.select_car_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |m| m.tables.draw_lists.select_car_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.select_car_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.select_car_picture
    }))],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    leaving: &[Step::Do(leaving)],
    ..Screen::new(
        |m| m.tables.menus.select_car.id(),
        |m| &m.screens.items.current[slot::SELECT_CAR],
    )
};
