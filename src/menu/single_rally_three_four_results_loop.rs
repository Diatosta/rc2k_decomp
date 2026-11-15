//! A screen titled "RESULTADOS" called by `menu::single_rally_three` and `menu::single_rally_four`.
//! Screen-specific part of `menu::screen_loop`: background `results1.png`; leaving is asked about;
//! left without the question, the highlight is not saved; its own 3D scene list is set before the
//! first frame only; around an action the highlight is kept in the saved highlight; the action run
//! is that of the item of another menu; on leaving the highlight is saved in the saved highlight,
//! not in the variable it is taken from (1999 slips kept as is).
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Asking, Callee, Frames, Keys, List, MenuSound, Screen, ScreenFlag, Step,
};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list,
            table: |_| DRAW_LISTS.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.single_rally_three_four_results_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.single_rally_three_four_results_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(ScreenFlag::EventOver, false)],
    set_up_first: &[
        Step::Call(Callee::RankStageTimes),
        Step::Call(Callee::ScoreFourPlayers),
    ],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.single_rally_three_four_results_picture
    }))],
    after: &[Step::Sound(MenuSound::None)],
    leave_saved_item: Some(slot::STAGE_RESULT),
    asking: Some(Asking {
        unasked_skips_save: true,
        ..Asking::DEFAULT
    }),
    keys: Keys {
        action_saved_item: Some(slot::STAGE_RESULT),
        action_menu: Some(|m| m.tables.menus.stage_result.id()),
        ..Keys::DEFAULT
    },
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |_| DRAW_LISTS.single_rally_three_four_results_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.single_rally_three_four_results_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.single_rally_three_four_results_overlay.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.single_rally_three_four_results.id(),
        slot::SINGLE_RALLY_THREE_FOUR_RESULTS,
    )
};
