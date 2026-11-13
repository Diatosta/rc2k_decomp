//! Player 1's controller set-up page (CONFIGURAÇÃO DO COMANDO, background `control.png`; Options >
//! Player details > Player 1 details > "configurar comando": the controller, force feedback, axis
//! mode). Screen-specific part of `menu::screen_loop`: background `control.png` through
//! `menu::load_background`; leaving is not asked about; its own 3D scene list is set before the
//! first frame only.
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Frames, List, Screen, Step, screen_entry};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list,
            table: |m| m.tables.draw_lists.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.player1_controller_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |m| m.tables.draw_lists.player1_controller_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.player1_controller_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Call(Callee::ApplyController)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.player1_controller_picture
    }))],
    after: &[Step::Call(Callee::ReleaseController)],
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |m| m.tables.draw_lists.player1_controller_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |m| m.tables.draw_lists.player1_controller_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |m| m.tables.draw_lists.player1_controller_overlay.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.player1_controller.id(),
        |m| &m.screens.items.current[slot::PLAYER1_CONTROLLER],
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::player1_controller_loop", &SCREEN)
}
