//! `menu::new_simulation_loop` (0x4A99AA): the loop of the screen that starts a new simulation.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `sral.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "NOVA SIMULAÇÃO", not yet a visit.
//! The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Entry, List, Screen, Step, carousel, screen_entry};

fn before(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.items.current[slot::NEW_SIMULATION]
        .set(screens.items.first[slot::NEW_SIMULATION].get());
    screens.event.event_over.set(false);
    Ok(())
}

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x2u32);
    carousel(None)?;
    let item = screens.items.current[slot::NEW_SIMULATION].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));

    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.new_simulation_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.new_simulation_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.new_simulation_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Do(before)],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.new_simulation_picture
    }))],
    after_first_frame: &[
        Step::WriteScene(
            |_| &crate::scene::polygon_work::front_end().window.view_offset[0],
            0x7,
        ),
        Step::WriteScene(
            |_| &crate::scene::polygon_work::front_end().window.view_offset[1],
            0xA,
        ),
        Step::WriteScene(
            |_| &crate::scene::polygon_work::front_end().window.view_zoom,
            0x100,
        ),
        Step::Call(Callee::PrepareScene),
    ],
    ..Screen::new(
        |m| m.tables.menus.new_simulation.id(),
        |m| &m.screens.items.current[slot::NEW_SIMULATION],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::new_simulation_loop", &SCREEN)
}
