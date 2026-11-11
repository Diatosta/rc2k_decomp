//! `menu::time_trial_results_loop` (0x48DB7D): the results screen of the time trial, by its
//! title "RESULTADOS" and its callers (named from its title string and callers; the original's name is lost)
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `results1.png`; leaving is asked about; left
//! without the question, the highlight is not saved; its own 3D list goes to `g_sceneList`
//! before the first frame only.
//! The name is a reading from the title string of its menu, "RESULTADOS", not yet a visit.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, Frames, List, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list, // g_unreadList
            table: |m| m.tables.draw_lists.unread_758b72.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.draw_758e96.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.overlay_7592d2.id(),
            run: false,
        },
    ],
    before: &[Step::Write(|m| &m.screens.event.event_done, 0x0)],
    set_up_first: &[
        Step::Call(Callee::RankStageTimes),
        Step::Call(Callee::ScoreFourPlayers),
    ],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.time_trial_results_picture
    }))],
    after: &[Step::Write(|m| &m.screens.option_work.menu_sound, 0x6F)],
    asking: Some(Asking {
        unasked_skips_save: true,
        ..Asking::DEFAULT
    }),
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list, // g_sceneList
                table: |m| m.tables.draw_lists.time_trial_results_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list, // g_drawList
                table: |m| m.tables.draw_lists.draw_758e96.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
                table: |m| m.tables.draw_lists.overlay_7592d2.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.time_trial_results.id(),
        |m| &m.screens.items.current[slot::TIME_TRIAL_RESULTS],
    )
};
