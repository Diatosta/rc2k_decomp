//! `menu::single_rally_two_results_loop` (0x48584B): a results screen, by its title "RESULTADOS" (a
//! reading)
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `results1.png`; leaving is asked about; left
//! without the question, the highlight is not saved; its own 3D list goes to `g_sceneList`
//! before the first frame only; around an action the highlight is kept in the saved highlight; the
//! action run is that of the item of another menu, 0x750ACC (the original's slip).
//! The name is a reading from the title string of its menu, "RESULTADOS", not yet a visit.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, Frames, Keys, List, MenuSound, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list, // g_unreadList
            table: |m| m.tables.draw_lists.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.single_rally_two_results_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.single_rally_two_results_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Write(|m| &m.screens.event.event_over, 0x0)],
    set_up_first: &[Step::Call(Callee::ScoreTwoPlayers)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.single_rally_two_results_picture
    }))],
    after: &[Step::Sound(MenuSound::None)],
    asking: Some(Asking {
        unasked_skips_save: true,
        ..Asking::DEFAULT
    }),
    keys: Keys {
        action_saved_item: Some(|m| &m.screens.items.current[slot::STAGE_RESULT]),
        action_menu: Some(|m| m.tables.menus.stage_result.id()),
        ..Keys::DEFAULT
    },
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list, // g_sceneList
                table: |m| m.tables.draw_lists.single_rally_two_results_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list, // g_drawList
                table: |m| m.tables.draw_lists.single_rally_two_results_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
                table: |m| m.tables.draw_lists.single_rally_two_results_overlay.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.single_rally_two_results.id(),
        |m| &m.screens.items.current[slot::SINGLE_RALLY_TWO_RESULTS],
    )
};
