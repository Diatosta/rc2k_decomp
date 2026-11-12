//! `menu::car_specification_loop` (0x47A652): the screen titled "ESPECIFICAÇÕES DO CARRO".
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is not asked about; it has a step of its own once
//! a frame; one call comes after the registers are put back; the steps that are more than a
//! call or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "ESPECIFICAÇÕES DO CARRO", not yet
//! a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Frames, List, Screen, Step, carousel, prepare_scene, screen_entry,
};

fn before(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    menus
        .screens
        .event
        .shown_car
        .set(menus.play.cheats.kept_car.get());
    menus.screens.event.car_page.set(0x0u32);

    Ok(())
}

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    entry.set_view(-160, -100, 0x100);
    prepare_scene(entry.scene);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0xBu32);
    carousel(None)?;
    let item = screens.items.current[slot::CAR_SPECIFICATION].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    screens
        .state
        .carousel
        .front
        .set(screens.event.shown_car.get());
    Ok(())
}

fn after(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    menus
        .play
        .cheats
        .kept_car
        .set(menus.screens.event.shown_car.get());

    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.car_specification_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.car_specification_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |m| m.tables.draw_lists.car_specification_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.car_specification_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Do(before)],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[
        Step::Call(Callee::LoadCarBackground),
        Step::Call(Callee::PlayCarStream),
    ],
    after: &[Step::Do(after)],
    after_restored: &[Step::Call(Callee::StopStream)],
    frames: Frames {
        each_frame: &[Step::Call(Callee::StepCarStream)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.car_specification.id(),
        |m| &m.screens.items.current[slot::CAR_SPECIFICATION],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::car_specification_loop", &SCREEN)
}
