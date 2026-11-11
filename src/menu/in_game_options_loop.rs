//! `menu::in_game_options_loop` (0x47D8F7): the loop of the options menu opened during an
//! event.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `opts.png`; leaving is not asked about; the
//! Escape flag is looked at at the top of every frame; the steps that are more than a call or a
//! store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "OPÇÕES NO JOGO", not yet a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Keys, List, Screen, Step, carousel, prepare_scene, screen_entry,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    entry.set_view(-156, 39, 0x140);
    prepare_scene(entry.scene);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x11u32);
    carousel(None)?;
    let item = screens.items.current[slot::IN_GAME_OPTIONS].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.in_game_options_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.in_game_options_list.from(2),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.in_game_options_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(
        |m| &m.screens.draw_work.escape_pressed,
        false,
    )],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.in_game_options_picture
    }))],
    keys: Keys {
        escape_leaves: true,
        ..Keys::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.in_game_options.id(),
        |m| &m.screens.items.current[slot::IN_GAME_OPTIONS],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::in_game_options_loop", &SCREEN)
}
