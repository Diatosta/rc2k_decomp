//! The start screen of the normal championship, by its title "NOVA SIMULAÇÃO" and its place in the
//! mode menu. Screen-specific part of `menu::screen_loop`: background `sral.png`; leaving is not
//! asked about; the next frame begins with the steps after the first frame.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, Frames, List, Screen, Step, carousel, prepare_scene, screen_entry,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    menus.tables.captions.left_by_key.set(false);
    menus.screens.flow.turning.set(0x1u32);
    menus.screens.state.carousel.menu.set(0x2u32);
    carousel(entry, None)?;
    let item = menus.screens.items.current[slot::CHAMPIONSHIP_NEW_SIMULATION].get();
    menus.screens.loop_flags.item.set(item);
    menus.screens.state.carousel.front.set(item.wrapping_sub(1));
    entry.set_view(7, 10, 0x100);
    prepare_scene(entry);
    Ok(())
}

fn after_first_frame(entry: &Entry) -> Result<Flow, Refused> {
    let tables = &entry.menus.tables;
    if tables.captions.left_by_key.is_set() {
        tables.captions.left_by_key.set(false);
        return Ok(Flow::SetUp);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.championship_new_simulation_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.championship_new_simulation_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.championship_new_simulation_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(|m| &m.screens.event.event_over, false)],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.championship_new_simulation_picture
    }))],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    frames: Frames {
        next_frame_repeats_first: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.championship_new_simulation.id(),
        |m| &m.screens.items.current[slot::CHAMPIONSHIP_NEW_SIMULATION],
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::championship_new_simulation_loop", &SCREEN)
}
