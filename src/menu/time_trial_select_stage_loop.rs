//! `menu::time_trial_select_stage_loop` (0x48BFCB): the stage selection of the time trial, by
//! its title "ESCOLHA ETAPA" and its callers (a reading)
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; left without the question, the
//! highlight is not saved.
//! The name is a reading from the title string of its menu, "ESCOLHA ETAPA", not yet a visit.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, FrameRoutine, List, Screen, Step};

pub const SCREEN: Screen = Screen {
    menu: |m| m.tables.menus.time_trial_select_stage.id(),
    saved_item: |m| &m.screens.items.current[slot::TIME_TRIAL_SELECT_STAGE],
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
            pointer: |m| &m.screens.flow.setup_list, // dword_755500
            table: |m| m.tables.draw_lists.time_trial_select_stage_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.time_trial_select_stage_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[Step::Write(|m| &m.tables.pages.yes_no_answer, 0x0)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[Step::Write(|m| &m.play.champ_view.stage_mode, 0x1)],
    background: &[Step::Call(Callee::DrawStageMap)],
    after_first_frame: &[],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: true,
    after: &[Step::Write(|m| &m.screens.option_work.menu_sound, 0x6F)],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: false,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: false,
    before_asking: &[],
    ask_unless: None,
    leaving: &[],
    leave_saved_item: None,
    keys_menu: None,
    after_restored: &[],
    result: None,
    late_highlight: false,
    before_first_frame: &[],
    before_frame: &[],
    after_blink: &[],
    restart_action_check: false,
    always_asks: false,
};
