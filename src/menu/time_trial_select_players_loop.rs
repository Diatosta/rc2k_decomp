//! `menu::time_trial_select_players_loop` (0x48BCDC): the loop of the screen where the number of players
//! is chosen.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `arcade1.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! Screen: the player count of the time trial, "ESCOLHA NÚMERO DE JOGADORES".
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, List, MenuSound, Screen, Step, carousel, prepare_scene, screen_entry,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    entry.set_view(7, 10, 0x100);
    prepare_scene(entry);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x2u32);
    carousel(entry, None)?;
    let item = screens.items.current[slot::TIME_TRIAL_SELECT_PLAYERS].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.time_trial_select_players_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.time_trial_select_players_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.time_trial_select_players_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(|m| &m.screens.event.event_over, false)],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.time_trial_select_players_picture
    }))],
    after: &[Step::Sound(MenuSound::None)],
    ..Screen::new(
        |m| m.tables.menus.time_trial_select_players.id(),
        |m| &m.screens.items.current[slot::TIME_TRIAL_SELECT_PLAYERS],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::time_trial_select_players_loop", &SCREEN)
}
