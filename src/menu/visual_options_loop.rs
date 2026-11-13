//! The loop of the visual options screen. Screen-specific part of `menu::screen_loop`: background
//! `visual.png` through `menu::load_background` (a failed load is fatal error 0x80712345); leaving
//! is not asked about.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, List, Screen, Step, screen_entry};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.visual_options_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.visual_options_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.visual_options_overlay.id(),
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.visual_options_picture
    }))],
    ..Screen::new(
        |m| m.tables.menus.visual_options.id(),
        |m| &m.screens.items.current[slot::VISUAL_OPTIONS],
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::visual_options_loop", &SCREEN)
}
