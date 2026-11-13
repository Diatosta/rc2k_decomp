//! The stage selection of the time trial, by its title "ESCOLHA ETAPA" and its callers.
//! Screen-specific part of `menu::screen_loop`: leaving is asked about; left without the
//! question, the highlight is not saved.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, MenuSound, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.time_trial_select_stage_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.time_trial_select_stage_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |m| m.tables.draw_lists.time_trial_select_stage_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.time_trial_select_stage_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Write(|m| &m.tables.pages.yes_no_answer, 0x0)],
    after_lists: &[Step::Write(|m| &m.play.champ_view.stage_mode, 0x1)],
    background: &[Step::Call(Callee::DrawStageMap)],
    after: &[Step::Sound(MenuSound::None)],
    asking: Some(Asking {
        unasked_skips_save: true,
        ..Asking::DEFAULT
    }),
    ..Screen::new(
        |m| m.tables.menus.time_trial_select_stage.id(),
        |m| &m.screens.items.current[slot::TIME_TRIAL_SELECT_STAGE],
    )
};
