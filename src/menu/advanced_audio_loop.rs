//! The loop of the advanced audio set-up screen. Screen-specific part of `menu::screen_loop`:
//! background `audio.png` through `menu::load_background`; leaving is not asked about.
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, List, Screen, Step, screen_entry};
use crate::menu::texts_store::texts;

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.advanced_audio_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.advanced_audio_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.advanced_audio_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.advanced_audio_overlay.id(),
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().advanced_audio_picture
    }))],
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.advanced_audio.id(),
        |m| &m.screens.items.current[slot::ADVANCED_AUDIO],
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, &SCREEN)
}
