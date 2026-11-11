//! `menu::stage_result_loop` (0x48558D): the loop of a stage's result screen. Seen after a
//! single race's stage (owner, 2026-10-22: the standalone reached it there); earlier
//! called a time trial's, wrongly.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `results1.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is asked about;
//! its own 3D list goes to `g_sceneList` before the first frame only.
//! Screen: the one player's result after a stage, "RESULTADOS": the rally and
//! stage, then "piloto", "tempo" and "velocidade máxima" (owner's screenshot from a Rust-run
//! visit, 2026-10-11). Named `screen_48558d_loop` until then.
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
            table: |m| m.tables.draw_lists.draw_758e7e.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.overlay_7592c2.id(),
            run: false,
        },
    ],
    before: &[Step::Write(|m| &m.screens.event.event_done, 0x0)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.stage_result_picture
    }))],
    after: &[Step::Write(|m| &m.screens.option_work.menu_sound, 0x6F)],
    asking: Some(Asking { ..Asking::DEFAULT }),
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list, // g_sceneList
                table: |m| m.tables.draw_lists.stage_result_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list, // g_drawList
                table: |m| m.tables.draw_lists.draw_758e7e.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
                table: |m| m.tables.draw_lists.overlay_7592c2.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.stage_result.id(),
        |m| &m.screens.items.current[slot::STAGE_RESULT],
    )
};
