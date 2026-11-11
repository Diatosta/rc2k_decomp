//! `menu::arcade_results_loop` (0x4891EF): a results screen, by its title "RESULTADOS" (a
//! reading)
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`), as
//! extracted from the disassembly: background `results1.png`; leaving is asked about; left without
//! the question, the highlight is not saved; its own 3D list goes to `g_sceneList` before the first
//! frame only. Named by its address until then.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, Frames, List, MenuSound, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list, // g_unreadList
            table: |m| m.tables.draw_lists.unread_758b72.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.draw_758e76.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.overlay_7592be.id(),
            run: false,
        },
    ],
    before: &[Step::Write(|m| &m.screens.event.event_over, 0x0)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.arcade_results_picture
    }))],
    after: &[Step::Sound(MenuSound::None)],
    asking: Some(Asking {
        unasked_skips_save: true,
        ..Asking::DEFAULT
    }),
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list, // g_sceneList
                table: |m| m.tables.draw_lists.arcade_results_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list, // g_drawList
                table: |m| m.tables.draw_lists.draw_758e76.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
                table: |m| m.tables.draw_lists.overlay_7592be.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.arcade_results.id(),
        |m| &m.screens.items.current[slot::ARCADE_RESULTS],
    )
};
