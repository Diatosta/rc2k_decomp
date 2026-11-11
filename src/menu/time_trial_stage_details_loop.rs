//! `menu::time_trial_stage_details_loop` (0x48F1F4): the loop of the stage details screen of a
//! time trial.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: leaving is not asked about.
//! A reading, not yet a visit: its draw list 0x758CFE is the one of
//! `screens::draw_time_trial_stage_info`, a confirmed screen; it sets `dword_755700` to 1 first.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, List, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.time_trial_stage_details_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.time_trial_stage_details_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
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
