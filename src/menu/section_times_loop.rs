//! `menu::unused_section_times_loop_2` (the same but for its scene list: `_1`): the ten best
//! times of a section (`screens::draw_section_best_times`), background `results1.png`. Nothing
//! in the menus reaches either; reached here from the debug UI's "Screens" tab
//! (`debug::screens`), which says the section (`EventState.section`).
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_loop::{Callee, List, Screen, Step};
use crate::menu::texts_store::texts;

/// `ScreenItems.current[78]`'s slot, the item kept between visits.
const SLOT: usize = (0x747798 - 0x747660) / 4;

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.section_times_scene_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.section_times_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.section_times_overlay_list.id(),
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().section_times_png
    }))],
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.section_times_menu.id(),
        SLOT,
    )
};
