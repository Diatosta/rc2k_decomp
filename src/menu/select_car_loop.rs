//! `menu::select_car_loop`: the car selection screen, which the starters of
//! every kind of event run (the car select menu).
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`): background `cspecs.png`; leaving is not asked about; it has steps of its own
//! first on leaving.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Menus;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Flow, List, Screen, SetUp, Step, quitting};
use crate::system::owner::System;

fn set_up_first(set_up: &SetUp) -> Result<(), Refused> {
    let menus = set_up.menus;
    let screens = &menus.screens;
    // The current player's gearing, shown first.
    let current: u32 = screens.event.read().current_player;
    let Some(player) = crate::menu::player::player(menus, current) else {
        return Err(crate::errors::BadInput("a current player past the four").into());
    };
    screens.flow.write().shown_gearing = player.get().gearing;
    screens.flow.write().turning = 0x1u32;
    screens.carousel.write().menu = 0xDu32;
    set_up.carousel()?;
    let item = screens.items.current(slot::SELECT_CAR);
    screens.loop_flags.write().item = item;
    screens.carousel.write().front = item.wrapping_sub(1);
    screens.flow.write().turning = 0x0u32;
    screens.carousel.write().front = 0x0u32;
    screens.event.write().shown.car = 0x0u32;
    crate::menu::car_choice::find_player_car(menus)?;
    set_up.set_view(7, -50, 0x100);
    set_up.prepare_scene();
    Ok(())
}

fn after_first_frame(system: &System) -> Result<Flow, Refused> {
    if quitting(system) as u32 == 0x1 {
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

fn leaving(menus: &Menus) -> Result<(), Refused> {
    if menus.tables.captions.read().left_by_key {
        crate::menu::go_back::go_back(menus);
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.select_car_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.select_car_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.select_car_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.select_car_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::DoSetUp(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.select_car_picture
    }))],
    after_first_frame: &[Step::DoSystemFlow(after_first_frame)],
    leaving: &[Step::DoMenus(leaving)],
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.select_car.id(),
        slot::SELECT_CAR,
    )
};
