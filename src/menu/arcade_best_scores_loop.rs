//! `menu::arcade_best_scores_loop` (0x48A69E): a best scores screen, by its title "MELHORES
//! CLASSIFICAÇÕES" (named from its title string and callers; the original's name is lost)
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `bascores.png`; leaving is asked about; left
//! without the question, the highlight is not saved; its own 3D list goes to `g_sceneList`
//! before the first frame only.
//! The name is a reading from the title string of its menu, "MELHORES CLASSIFICAÇÕES", not yet
//! a visit.
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
            table: |m| m.tables.draw_lists.draw_758d32.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.overlay_759222.id(),
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.arcade_best_scores_picture
    }))],
    asking: Some(Asking {
        unasked_skips_save: true,
        ..Asking::DEFAULT
    }),
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list, // g_sceneList
                table: |m| m.tables.draw_lists.arcade_best_scores_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list, // g_drawList
                table: |m| m.tables.draw_lists.draw_758d32.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
                table: |m| m.tables.draw_lists.overlay_759222.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.arcade_best_scores.id(),
        |m| &m.screens.items.current[slot::ARCADE_BEST_SCORES],
    )
};
