//! `menu::unused_cspecs_loop`: a car specification screen with the carousel of
//! menu 0xC, background `cspecs.png`. Nothing in the menus reaches it; reached from the
//! debug UI's "Screens" tab (`debug::screens`). What it was for is not known; a reading.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Screens;
use crate::menu::screen_loop::{Callee, List, Screen, SetUp, Step};

/// `ScreenItems.current[33]`'s slot, the item kept between visits.
const SLOT: usize = 33;

fn before(screens: &Screens) -> Result<(), Refused> {
    screens.items.current[SLOT].set(screens.items.first[SLOT].get());
    Ok(())
}

// The scene is prepared after the carousel, so the front car is not left white.
fn set_up_first(set_up: &SetUp) -> Result<(), Refused> {
    let screens = &set_up.menus.screens;
    screens.flow.write().turning = 0x1u32;
    screens.carousel.menu.set(0xCu32);
    set_up.carousel_with(None)?;
    screens.loop_flags.write().item = screens.items.current[SLOT].get();
    screens.flow.write().turning = 0x0u32;
    screens.carousel.front.set(0x0u32);
    screens.event.write().shown.car = 0x0u32;
    set_up.set_view(7, -50, 0x100);
    set_up.prepare_scene();
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.cspecs_scene_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.cspecs_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.cspecs_setup_list.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.cspecs_overlay_list.id(),
            run: false,
        },
    ],
    before: &[Step::DoScreens(before)],
    set_up_first: &[Step::DoSetUp(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| &m.texts.cspecs_png))],
    ..Screen::new(module_path!(), |m| m.tables.menus.cspecs_menu.id(), SLOT)
};
