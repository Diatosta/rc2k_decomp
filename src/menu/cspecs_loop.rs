//! `menu::unused_cspecs_loop` (0x4AA1A3): a car specification screen with the carousel of
//! menu 0xC, background `cspecs.png`. Nothing in RAL.EXE reaches it; reached here from the
//! debug UI's "Screens" tab (`debug::screens`). What it was for is not known; a reading.

use crate::errors::Refused;
use crate::menu::screen_loop::{Callee, Entry, List, Screen, Step, carousel, prepare_scene};

/// `unk_7476E4`'s slot, the item kept between visits.
const SLOT: usize = (0x7476E4 - 0x747660) / 4;

fn before(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.items.current[SLOT].set(screens.items.first[SLOT].get());
    Ok(())
}

// The original prepares the scene before the carousel, which left the front car white
// (seen 2026-10-30): here as `menu::select_car_loop`, the scene last.
fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0xCu32);
    carousel(None)?;
    screens
        .loop_flags
        .item
        .set(screens.items.current[SLOT].get());
    screens.flow.turning.set(0x0u32);
    screens.state.carousel.front.set(0x0u32);
    screens.event.shown_car.set(0x0u32);
    entry.set_view(7, -50, 0x100);
    prepare_scene(entry.scene);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.cspecs_scene_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.cspecs_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |m| m.tables.draw_lists.cspecs_setup_list.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.cspecs_overlay_list.id(),
            run: false,
        },
    ],
    before: &[Step::Do(before)],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.cspecs_png
    }))],
    ..Screen::new(
        |m| m.tables.menus.cspecs_menu.id(),
        |m| &m.screens.items.current[SLOT],
    )
};
