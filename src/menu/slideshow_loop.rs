//! The loop of the slideshow screen. Screen-specific part of `menu::screen_loop`: leaving is not
//! asked about; it has a step of its own once a frame.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Frames, List, Screen, Step, screen_entry};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.slideshow_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.slideshow_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |m| m.tables.draw_lists.slideshow_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.slideshow_overlay.id(),
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadSlideshowPicture)],
    frames: Frames {
        each_frame: &[Step::Call(Callee::StepCarStream)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.slideshow.id(),
        |m| &m.screens.items.current[slot::SLIDESHOW],
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::slideshow_loop", &SCREEN)
}
