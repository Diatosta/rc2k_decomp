//! `menu::single_rally_overall_results_loop` (0x483163): the loop of a results screen.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own
//! (`SCREEN`), as extracted from the disassembly: background `results1.png` through
//! `menu::load_background` (carry afterwards = fatal error 0x80712345); leaving is not asked
//! about; its own 3D list goes to `g_sceneList` before the first frame only.
//! Which screen it is has not been confirmed, hence the name. Its title string: "RESULTADOS
//! GERAIS".
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Frames, List, MenuSound, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list, // g_unreadList
            table: |m| m.tables.draw_lists.unread_758b72.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.draw_758ece.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.overlay_7592fe.id(),
            run: false,
        },
    ],
    before: &[Step::Write(|m| &m.screens.event.event_over, 0x0)],
    set_up_first: &[Step::Call(Callee::RankTotals)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.single_rally_overall_results_picture
    }))],
    after: &[Step::Sound(MenuSound::None)],
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list, // g_sceneList
                table: |m| m.tables.draw_lists.single_rally_overall_results_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list, // g_drawList
                table: |m| m.tables.draw_lists.draw_758ece.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
                table: |m| m.tables.draw_lists.overlay_7592fe.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.single_rally_overall_results.id(),
        |m| &m.screens.items.current[slot::SINGLE_RALLY_OVERALL_RESULTS],
    )
};
