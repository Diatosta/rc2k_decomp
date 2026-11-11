//! `menu::arcade_select_level_loop` (0x487999): the mode menu's third entry, by its title
//! "ESCOLHA NÍVEL" (named from its title string and callers; the original's name is lost)
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `Selarc.png`; leaving is not asked about; the
//! next frame begins with the steps after the first frame; the steps that are more than a call
//! or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "ESCOLHA NÍVEL", not yet a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, Frames, List, Screen, Step, carousel, prepare_scene, screen_entry,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    crate::scene::polygon_work::front_end().window.view_offset[0].set(0x7u32);
    crate::scene::polygon_work::front_end().window.view_offset[1].set(0xAu32);
    crate::scene::polygon_work::front_end()
        .window
        .view_zoom
        .set(0x100u32);
    prepare_scene(entry.scene);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x5u32);
    carousel(None)?;
    let item = screens.items.current[slot::ARCADE_SELECT_LEVEL].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    Ok(())
}

fn after_first_frame(entry: &Entry) -> Result<Flow, Refused> {
    let tables = &entry.menus.tables;
    if tables.captions.screen_left.get() == 0x1 {
        tables.captions.screen_left.set(0x0u32);
        return Ok(Flow::SetUp);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.arcade_select_level_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.arcade_select_level_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.arcade_select_level_overlay.id(),
            run: false,
        },
    ],
    before: &[
        Step::Write(|m| &m.tables.captions.screen_left, 0x0),
        Step::Write(|m| &m.screens.event.event_done, 0x0),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.arcade_select_level_picture
    }))],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    after: &[Step::Write(|m| &m.screens.option_work.menu_sound, 0x6F)],
    frames: Frames {
        next_frame_repeats_first: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.arcade_select_level.id(),
        |m| &m.screens.items.current[slot::ARCADE_SELECT_LEVEL],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::arcade_select_level_loop", &SCREEN)
}
