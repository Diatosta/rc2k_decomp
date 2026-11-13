//! A screen titled "RESULTADOS" called by `menu::single_rally_three` and `menu::single_rally_four`.
//! Screen-specific part of `menu::screen_loop`: background `results1.png`; leaving is asked about;
//! left without the question, the highlight is not saved; its own 3D scene list is set before the
//! first frame only; around an action the highlight is kept in the saved highlight; the action run
//! is that of the item of another menu; on leaving the highlight is saved in the saved highlight,
//! not in the variable it is taken from; the keys are handled with another menu, in
//! `menu_texts.keys_menu` (all 1999 slips kept as is).
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, Frames, Keys, List, MenuSound, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list,
            table: |m| m.tables.draw_lists.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| {
                m.tables
                    .draw_lists
                    .single_rally_three_four_results_draw
                    .id()
            },
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| {
                m.tables
                    .draw_lists
                    .single_rally_three_four_results_overlay
                    .id()
            },
            run: false,
        },
    ],
    before: &[Step::WriteFlag(|m| &m.screens.event.event_over, false)],
    set_up_first: &[
        Step::Call(Callee::RankStageTimes),
        Step::Call(Callee::ScoreFourPlayers),
    ],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.single_rally_three_four_results_picture
    }))],
    after: &[Step::Sound(MenuSound::None)],
    leave_saved_item: Some(|m| &m.screens.items.current[slot::STAGE_RESULT]),
    asking: Some(Asking {
        unasked_skips_save: true,
        ..Asking::DEFAULT
    }),
    keys: Keys {
        menu: Some(|m| m.tables.menus.stage_result.id()),
        action_saved_item: Some(|m| &m.screens.items.current[slot::STAGE_RESULT]),
        action_menu: Some(|m| m.tables.menus.stage_result.id()),
        ..Keys::DEFAULT
    },
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |m| {
                    m.tables
                        .draw_lists
                        .single_rally_three_four_results_scene
                        .id()
                },
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |m| {
                    m.tables
                        .draw_lists
                        .single_rally_three_four_results_draw
                        .id()
                },
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |m| {
                    m.tables
                        .draw_lists
                        .single_rally_three_four_results_overlay
                        .id()
                },
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.single_rally_three_four_results.id(),
        |m| &m.screens.items.current[slot::SINGLE_RALLY_THREE_FOUR_RESULTS],
    )
};
