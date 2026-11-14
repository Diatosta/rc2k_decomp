//! A screen titled "ETAPA SEGUINTE" under the mode menu's third entry. Screen-specific part of
//! `menu::screen_loop`: leaving is asked about; left without the question, the highlight is not
//! saved.
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, MenuSound, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.arcade_next_stage_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.arcade_next_stage_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.arcade_next_stage_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Write(|m| &m.tables.pages.yes_no_answer, 0x0)],
    after_lists: &[Step::Write(|m| &m.play.champ_view.stage_mode, 0x2)],
    background: &[Step::Call(Callee::DrawStageMap)],
    after: &[Step::Sound(MenuSound::None)],
    asking: Some(Asking {
        unasked_skips_save: true,
        ..Asking::DEFAULT
    }),
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.arcade_next_stage.id(),
        |m| &m.screens.items.current[slot::ARCADE_NEXT_STAGE],
    )
};
