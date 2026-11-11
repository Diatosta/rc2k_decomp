//! `menu::time_trial_select_stage_loop` (0x48BFCB): the stage selection of the time trial, by
//! its title "ESCOLHA ETAPA" and its callers (named from its title string and callers; the original's name is lost)
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; left without the question, the
//! highlight is not saved.
//! The name is a reading from the title string of its menu, "ESCOLHA ETAPA", not yet a visit.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, MenuSound, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.time_trial_select_stage_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.time_trial_select_stage_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |m| m.tables.draw_lists.time_trial_select_stage_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
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
