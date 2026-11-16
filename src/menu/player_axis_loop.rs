//! The loop of one controller axis's page, one page for the four players (the player is the
//! current one, the axis the one the axes list picked). Screen-specific part of
//! `menu::screen_loop`: the controller page's background, the controller kept applied for the
//! live bar, and its own draw list (`screens::axes_page`).

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Frames, List, Screen, Step, screen_entry};

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list,
            table: |_| DRAW_LISTS.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.player_axis_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.player_buttons_overlay[0].id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Call(Callee::ApplyController)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.player1_buttons_picture
    }))],
    after: &[Step::Call(Callee::ReleaseController)],
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |_| DRAW_LISTS.player_buttons_scene[0].id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.player_axis_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.player_buttons_overlay[0].id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.player_axis.id(),
        slot::PLAYER_AXIS,
    )
};

/// Enters the axes page for the current player.
pub fn run(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    screen_entry(o, ScreenId::PlayerAxis)
}
