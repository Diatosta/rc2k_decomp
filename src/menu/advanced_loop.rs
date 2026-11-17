//! The loop of the advanced page (our own: the switches of `var\ral.cfg`, the drawing threads and
//! the renderer). Screen-specific part of `menu::screen_loop`: the graphics page's background,
//! scene and overlay, and its own draw list (`screens::advanced_page`); leaving is not asked
//! about, and the changes are applied by its "apply" item (`menu::advanced_items`).
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, List, Screen, Step, screen_entry};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.visual_options_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.advanced_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.visual_options_overlay.id(),
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.visual_options_picture
    }))],
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.advanced.id(),
        slot::ADVANCED,
    )
};

/// Opens the page, showing the switches as they are now.
pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    crate::menu::advanced_items::forget_changes();
    screen_entry(o, ScreenId::Advanced)
}
