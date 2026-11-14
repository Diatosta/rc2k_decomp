//! The loop of the "leave?" question that `menu::confirm_leaving` puts. Screen-specific part of
//! `menu::screen_loop`: background `mainscr.png` through `menu::load_background`; leaving is not
//! asked about; its own 3D scene list is set before the first frame only.
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Frames, List, Screen, ScreenFlag, Step};
use crate::menu::texts_store::texts;

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list,
            table: |_| DRAW_LISTS.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.leave_question_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.leave_question_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::WriteFlag(ScreenFlag::Asking, true)],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().leave_question_picture
    }))],
    after: &[Step::WriteFlag(ScreenFlag::Asking, false)],
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |_| DRAW_LISTS.leave_question_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.leave_question_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.leave_question_overlay.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.leave_question.id(),
        slot::LEAVE_QUESTION,
    )
};
