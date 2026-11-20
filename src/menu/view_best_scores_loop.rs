//! The loop of the best scores screen opened from the menus. Screen-specific part of
//! `menu::screen_loop`: background `bascores.png`; leaving is not asked about.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Screens;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, List, Screen, Step, log_line, screen_entry};

fn before(screens: &Screens) -> Result<(), Refused> {
    screens.items.set_current(
        slot::VIEW_BEST_SCORES,
        screens.items.first(slot::VIEW_BEST_SCORES),
    );
    Ok(())
}

fn set_up_first() -> Result<(), Refused> {
    log_line(c"BScoresScr: Start");
    Ok(())
}

fn after() -> Result<(), Refused> {
    log_line(c"BScoresScr: End");
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.view_best_scores_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.view_best_scores_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.view_best_scores_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.view_best_scores_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::DoScreens(before)],
    set_up_first: &[Step::DoPlain(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.view_best_scores_picture
    }))],
    after: &[Step::DoPlain(after)],
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.view_best_scores.id(),
        slot::VIEW_BEST_SCORES,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::ViewBestScores)
}
