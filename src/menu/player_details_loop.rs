//! `menu::player_details_loop` (0x4773D6): the loop of the player details screen.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `pdetc.png`; leaving is not asked about; after
//! an action it is left or set up again, nothing else; the steps that are more than a call or a
//! store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "PORMENORES DO JOGADOR", not yet a
//! visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    ActionCheck, Callee, Entry, Keys, List, Screen, Step, carousel, prepare_scene, screen_entry,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    entry.set_view(-156, 41, 0x140);
    prepare_scene(entry);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x8u32);
    carousel(entry, None)?;
    let item = screens.items.current[slot::PLAYER_DETAILS].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.player_details_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.player_details_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.player_details_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.player_details_picture
    }))],
    keys: Keys {
        action_check: ActionCheck::Short,
        ..Keys::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.player_details.id(),
        |m| &m.screens.items.current[slot::PLAYER_DETAILS],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::player_details_loop", &SCREEN)
}
