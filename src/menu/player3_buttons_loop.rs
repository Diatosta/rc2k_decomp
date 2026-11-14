//! Player 3's controller binding page (CONFIGURAÇÃO DO COMANDO, background `control.png`; "progamar
//! comando" on player 3's controller set-up page: accelerate, brake, steer left and right ... bound
//! to buttons and axes). Screen-specific part of `menu::screen_loop`: background `control.png`
//! through `menu::load_background` (a failed load is fatal error 0x80712345); leaving is not asked
//! about; its own 3D scene list is set before the first frame only.
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Frames, List, Screen, Step, screen_entry};
use crate::menu::texts_store::texts;

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list,
            table: |_| DRAW_LISTS.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.player3_buttons_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.player3_buttons_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.player3_buttons_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[
        Step::Call(Callee::ApplyController),
        Step::Call(Callee::AxisMoved(0x0)),
    ],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().player3_buttons_picture
    }))],
    after: &[Step::Call(Callee::ReleaseController)],
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |_| DRAW_LISTS.player3_buttons_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.player3_buttons_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.player3_buttons_overlay.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.player3_buttons.id(),
        slot::PLAYER3_BUTTONS,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::Player3Buttons)
}
