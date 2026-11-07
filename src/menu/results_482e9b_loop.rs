//! `menu::results_482e9b_loop` (0x482E9B): a screen titled "RESULTADOS" called by
//! `menu::single_rally_three` and `menu::single_rally_four`.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `results1.png`; leaving is asked about; left
//! without the question, the highlight is not saved; its own 3D list goes to `g_sceneList`
//! before the first frame only; around an action the highlight is kept in `dword_7477D8`; the
//! action run is that of the item of another menu, 0x750ACC (the original's slip); on leaving
//! the highlight is saved in `dword_7477D8`, not in the variable it is taken from (the
//! original's slip); the keys are handled with another menu, 0x750ACC, in `dword_731BBB` (the
//! original's slip).
//! The name is a reading from the title string of its menu, "RESULTADOS", not yet a visit.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, FrameRoutine, List, Screen, Step};

// ---- Rust version: `menu::screen_loop` with this ----

pub const SCREEN: Screen = Screen {
    menu: |m| m.tables.menus.results_482e9b.id(),
    saved_item: |m| &m.screens.items.current[slot::RESULTS_482E9B],
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list, // g_unreadList
            table: |m| m.tables.draw_lists.unread_758b72.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.draw_758e8e.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.overlay_7592ce.id(),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.results_482e9b_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.draw_758e8e.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.overlay_7592ce.id(),
            run: false,
        },
    ]),
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[Step::Write(|m| &m.screens.event.event_done, 0x0)],
    set_up_first: &[
        Step::Call(Callee::RankStageTimes),
        Step::Call(Callee::ScoreFourPlayers),
    ],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.results_482e9b_picture
    }))],
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
    action_saved_item: Some(|m| &m.screens.items.current[slot::STAGE_RESULT]),
    action_menu: Some(|m| m.tables.menus.stage_result.id()),
    no_quit_check: false,
    before_asking: &[],
    ask_unless: None,
    leaving: &[],
    leave_saved_item: Some(|m| &m.screens.items.current[slot::STAGE_RESULT]),
    keys_menu: Some(|m| m.tables.menus.stage_result.id()),
    after_restored: &[],
    result: None,
    late_highlight: false,
    before_first_frame: &[],
    before_frame: &[],
    after_blink: &[],
    restart_action_check: false,
    always_asks: false,
};
