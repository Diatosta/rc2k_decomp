//! `menu::inspection_area_loop` (0x47DBF3): the loop of the inspection area screen.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`), as
//! extracted from the disassembly: background `service.png` through `menu::load_background` (carry
//! afterwards = fatal error 0x80712345); leaving is asked about. A reading from the title string of
//! its menu, not yet a visit: "ÁREA DE INSPECÇÃO".
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, FrameRoutine, Frames, List, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.inspection_area_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.inspection_area_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.inspection_area_overlay.id(),
            run: false,
        },
    ],
    background: &[
        Step::Call(Callee::LoadPicture(|m| {
            &m.tables.texts.inspection_area_picture
        })),
        Step::CallOrFatal(Callee::ConvertPicture, 0x80712345),
    ],
    asking: Some(Asking { ..Asking::DEFAULT }),
    frames: Frames {
        later_callback: Some(FrameRoutine::Inspection),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.inspection_area.id(),
        |m| &m.screens.items.current[slot::INSPECTION_AREA],
    )
};
