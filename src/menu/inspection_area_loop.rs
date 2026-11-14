//! The loop of the inspection area screen. Screen-specific part of `menu::screen_loop`: background
//! `service.png` through `menu::load_background` (a failed load is fatal error 0x80712345); leaving
//! is asked about.
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, FrameRoutine, Frames, List, Screen, Step};
use crate::menu::texts_store::texts;

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.inspection_area_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.inspection_area_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.inspection_area_overlay.id(),
            run: false,
        },
    ],
    background: &[
        Step::Call(Callee::LoadPicture(|_| &texts().inspection_area_picture)),
        Step::CallOrFatal(Callee::ConvertPicture, 0x80712345),
    ],
    asking: Some(Asking { ..Asking::DEFAULT }),
    frames: Frames {
        later_callback: Some(FrameRoutine::Inspection),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.inspection_area.id(),
        slot::INSPECTION_AREA,
    )
};
