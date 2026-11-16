//! The screen titled "BEM-VINDO" that the championship's event loops show. Screen-specific part of
//! `menu::screen_loop`: leaving is asked about; something runs before the question is put.
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, Screen, ScreenFlag, ScreenVar, Step};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.welcome_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.welcome_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.welcome_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(ScreenFlag::EscapePressed, false)],
    set_up_first: &[Step::Write(ScreenVar::YesNoAnswer, 0x0)],
    background: &[Step::Call(Callee::LoadWelcomePicture)],
    after_first_frame: &[Step::Call(Callee::PlayRallySpeech)],
    after: &[Step::Call(Callee::StopStream)],
    asking: Some(Asking {
        before: &[Step::Call(Callee::StopStream)],
        ..Asking::DEFAULT
    }),
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.welcome.id(),
        slot::WELCOME,
    )
};
