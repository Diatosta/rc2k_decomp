//! `menu::welcome_loop` (0x4721F0): the screen titled "BEM-VINDO" that the championship's event
//! loops show.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; something runs before the
//! question is put.
//! The name is a reading from the title string of its menu, "BEM-VINDO", not yet a visit.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, Screen, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.welcome_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.welcome_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.welcome_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Write(|m| &m.screens.draw_work.escape_pressed, 0x0)],
    set_up_first: &[Step::Write(|m| &m.tables.pages.yes_no_answer, 0x0)],
    background: &[Step::Call(Callee::LoadWelcomePicture)],
    after_first_frame: &[Step::Call(Callee::PlayRallySpeech)],
    after: &[Step::Call(Callee::StopStream)],
    asking: Some(Asking {
        before: &[Step::Call(Callee::StopStream)],
        ..Asking::DEFAULT
    }),
    ..Screen::new(
        |m| m.tables.menus.welcome.id(),
        |m| &m.screens.items.current[slot::WELCOME],
    )
};
