//! The loop of the best times screen opened from the menus. Screen-specific part of
//! `menu::screen_loop`: background `btimes.png`; leaving is not asked about.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Screens;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, List, Screen, Step, screen_entry};

fn before(screens: &Screens) -> Result<(), Refused> {
    screens.items.current[slot::VIEW_BEST_TIMES]
        .set(screens.items.first[slot::VIEW_BEST_TIMES].get());
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.view_best_times_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.view_best_times_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.view_best_times_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.view_best_times_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::DoScreens(before)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.view_best_times_picture
    }))],
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.view_best_times.id(),
        slot::VIEW_BEST_TIMES,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::ViewBestTimes)
}
