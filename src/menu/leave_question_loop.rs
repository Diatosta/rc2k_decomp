//! The loop of the "leave?" question that `menu::confirm_leaving` puts. Screen-specific part of
//! `menu::screen_loop`: background `mainscr.png` through `menu::load_background`; leaving is not
//! asked about; its own 3D scene list is set before the first frame only.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Frames, List, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list,
            table: |m| m.tables.draw_lists.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.leave_question_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.leave_question_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::WriteFlag(|m| &m.screens.flow.asking, true)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.leave_question_picture
    }))],
    after: &[Step::WriteFlag(|m| &m.screens.flow.asking, false)],
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |m| m.tables.draw_lists.leave_question_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |m| m.tables.draw_lists.leave_question_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |m| m.tables.draw_lists.leave_question_overlay.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.leave_question.id(),
        |m| &m.screens.items.current[slot::LEAVE_QUESTION],
    )
};
