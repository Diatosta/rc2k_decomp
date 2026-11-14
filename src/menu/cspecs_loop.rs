//! `menu::unused_cspecs_loop`: a car specification screen with the carousel of
//! menu 0xC, background `cspecs.png`. Nothing in RAL.EXE reaches it; reached here from the
//! debug UI's "Screens" tab (`debug::screens`). What it was for is not known; a reading.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_loop::{Callee, Entry, List, Screen, Step, carousel, prepare_scene};
use crate::menu::texts_store::texts;

/// `ScreenItems.current[33]`'s slot, the item kept between visits.
const SLOT: usize = (0x7476E4 - 0x747660) / 4;

fn before(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.items.current[SLOT].set(screens.items.first[SLOT].get());
    Ok(())
}

// The scene is prepared after the carousel, so the front car is not left white.
fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0xCu32);
    carousel(entry, None)?;
    screens
        .loop_flags
        .item
        .set(screens.items.current[SLOT].get());
    screens.flow.turning.set(0x0u32);
    screens.state.carousel.front.set(0x0u32);
    screens.event.shown_car.set(0x0u32);
    entry.set_view(7, -50, 0x100);
    prepare_scene(entry);
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
    before: &[Step::Do(before)],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|_| &texts().cspecs_png))],
    ..Screen::new(module_path!(), |m| m.tables.menus.cspecs_menu.id(), SLOT)
};
