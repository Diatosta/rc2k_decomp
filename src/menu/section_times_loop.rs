//! `menu::unused_section_times_loop_2` (0x467392; the same but for its scene list:
//! `_1`, 0x466E50): the ten best times of a section (`screens::draw_section_best_times`),
//! background `results1.png`. Nothing in RAL.EXE reaches either; reached here from the debug
//! UI's "Screens" tab (`debug::screens`), which says the section (`EVENT_STATE.section`).
use crate::menu::screen_loop::{Callee, List, Screen, Step};

/// `unk_747798`'s slot, the item kept between visits.
const SLOT: usize = (0x747798 - 0x747660) / 4;

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.section_times_scene_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.section_times_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.section_times_overlay_list.id(),
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.section_times_png
    }))],
    ..Screen::new(
        |m| m.tables.menus.section_times_menu.id(),
        |m| &m.screens.items.current[SLOT],
    )
};
