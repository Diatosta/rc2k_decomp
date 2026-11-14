//! The loop of the fourth player's details screen. Screen-specific part of `menu::screen_loop`:
//! background `pdet1.png`; leaving is not asked about; its own 3D scene list is set before the
//! first frame only; it has a step of its own once a frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Entry, Flow, Frames, List, Screen, Step, screen_entry};
use crate::menu::texts_store::texts;

fn each_frame(entry: &Entry) -> Result<Flow, Refused> {
    let screens = &entry.menus.screens;
    if screens.option_work.typing.is_set() {
        Callee::EditPlayerName.run(entry)?;
        if screens.option_work.typing.is_set() {
            return Ok(Flow::NextFrame);
        }
        Callee::CheckCheatName.run(entry)?;
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list,
            table: |_| DRAW_LISTS.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.player4_details_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.player4_details_overlay.id(),
            run: false,
        },
    ],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().player4_details_picture
    }))],
    after_first_frame: &[Step::Write(|m| &m.screens.event.current_player, 0x3)],
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |_| DRAW_LISTS.player4_details_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.player4_details_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.player4_details_overlay.id(),
                run: false,
            },
        ]),
        each_frame: &[Step::DoFlow(each_frame)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.player4_details.id(),
        |m| &m.screens.items.current[slot::PLAYER4_DETAILS],
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, &SCREEN)
}
