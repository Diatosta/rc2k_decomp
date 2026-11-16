//! A second screen titled "LOAD CHAMPIONSHIP", reached through a table. Screen-specific part of
//! `menu::screen_loop`: leaving is not asked about; on leaving the highlight is saved in the saved
//! highlight, not in the variable it is taken from (a 1999 slip kept as is).

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{List, Screen, ScreenVar, SetUp, Step, screen_entry};

fn prepare(set_up: &SetUp) -> Result<(), Refused> {
    set_up.set_view(0, 0, 0x100);
    set_up.prepare_scene();
    set_up.set_clip_sides(0, 800);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.load_championship_table_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.load_championship_table_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.load_championship_table_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Write(ScreenVar::LeftService, 0x0)],
    prepare: &[Step::DoSetUp(prepare)],
    after: &[Step::Write(ScreenVar::LeftService, 0x1)],
    leave_saved_item: Some(slot::SAVE_SETUP),
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.load_championship_table.id(),
        slot::LOAD_CHAMPIONSHIP_TABLE,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::LoadChampionshipTable)
}
