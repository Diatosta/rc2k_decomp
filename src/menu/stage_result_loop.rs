//! The loop of a stage's result screen. Seen after a single race's stage. Screen-specific part of
//! `menu::screen_loop`: background `results1.png` through `menu::load_background` (a failed load is
//! fatal error 0x80712345); leaving is asked about; its own 3D scene list is set before the first
//! frame only. Screen: the one player's result after a stage, "RESULTADOS": the rally and stage,
//! then "piloto", "tempo" and "velocidade máxima".
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, Frames, List, MenuSound, Screen, Step};
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
            table: |_| DRAW_LISTS.stage_result_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.stage_result_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(|m| &m.screens.event.event_over, false)],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().stage_result_picture
    }))],
    after: &[Step::Sound(MenuSound::None)],
    asking: Some(Asking { ..Asking::DEFAULT }),
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |_| DRAW_LISTS.stage_result_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.stage_result_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.stage_result_overlay.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.stage_result.id(),
        |m| &m.screens.items.current[slot::STAGE_RESULT],
    )
};
