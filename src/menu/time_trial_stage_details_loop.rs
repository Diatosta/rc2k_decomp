//! The loop of the stage details screen of a time trial. Screen-specific part of
//! `menu::screen_loop`: it sets `champ_view.stage_mode` to 1 first; leaving is not asked about.
//! Its draw list is that of `screens::draw_time_trial_stage_info`.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, List, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.time_trial_stage_details_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.time_trial_stage_details_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.time_trial_stage_details_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Write(|m| &m.play.champ_view.stage_mode, 0x1)],
    background: &[Step::Call(Callee::DrawStageMap)],
    ..Screen::new(
        |m| m.tables.menus.time_trial_stage_details.id(),
        |m| &m.screens.items.current[slot::TIME_TRIAL_STAGE_DETAILS],
    )
};
