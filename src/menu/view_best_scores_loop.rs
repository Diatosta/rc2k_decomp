//! The loop of the best scores screen opened from the menus. Screen-specific part of
//! `menu::screen_loop`: background `bascores.png`; leaving is not asked about.

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
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.view_best_scores_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.view_best_scores_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |m| m.tables.draw_lists.view_best_scores_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
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
        module_path!(),
        |m| m.tables.menus.view_best_scores.id(),
        |m| &m.screens.items.current[slot::VIEW_BEST_SCORES],
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, &SCREEN)
}
