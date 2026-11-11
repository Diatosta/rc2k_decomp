//! `menu::view_best_scores_loop` (0x48EF2E): the loop of the best scores screen opened from the
//! menus.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `bascores.png`; leaving is not asked about;
//! the steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "MELHORES CLASSIFICAÇÕES", not yet
//! a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Entry, List, Screen, Step, log_line, screen_entry};

fn before(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.items.current[slot::VIEW_BEST_SCORES]
        .set(screens.items.first[slot::VIEW_BEST_SCORES].get());
    Ok(())
}

fn set_up_first(_: &Entry) -> Result<(), Refused> {
    log_line(c"BScoresScr: Start");
    Ok(())
}

fn after(_: &Entry) -> Result<(), Refused> {
    log_line(c"BScoresScr: End");
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.view_best_scores_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.view_best_scores_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |m| m.tables.draw_lists.view_best_scores_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.view_best_scores_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Do(before)],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.view_best_scores_picture
    }))],
    after: &[Step::Do(after)],
    ..Screen::new(
        |m| m.tables.menus.view_best_scores.id(),
        |m| &m.screens.items.current[slot::VIEW_BEST_SCORES],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run(o: crate::owners::Owners<'_>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::view_best_scores_loop", &SCREEN)
}
