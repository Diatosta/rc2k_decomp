//! The stage selection of the time trial, by its title "SELECT STAGE".
//! Screen-specific part of `menu::screen_loop`: leaving is asked about; left without the
//! question, the highlight is not saved.
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, MenuSound, Screen, ScreenVar, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.time_trial_select_stage_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.time_trial_select_stage_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.time_trial_select_stage_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.time_trial_select_stage_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Write(ScreenVar::YesNoAnswer, 0x0)],
    after_lists: &[Step::Write(ScreenVar::StageMode, 0x1)],
    background: &[Step::Call(Callee::DrawStageMap)],
    after: &[Step::Sound(MenuSound::None)],
    asking: Some(Asking {
        unasked_skips_save: true,
        ..Asking::DEFAULT
    }),
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.time_trial_select_stage.id(),
        slot::TIME_TRIAL_SELECT_STAGE,
    )
};
