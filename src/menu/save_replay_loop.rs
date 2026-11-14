//! The replay naming screen "GRAVAR REPETIÇÃO"; returns the slot the replay goes to.
//! Screen-specific part of `menu::screen_loop`: background `loadsv.png`; leaving is not asked
//! about; its own 3D scene list is set before the first frame only; the Escape flag is looked at
//! at the top of every frame; it has a step of its own once a frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Menus;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Entry, Flow, Frames, Keys, List, Screen, ScreenFlag, Step};
use crate::menu::texts_store::texts;

fn each_frame(entry: &Entry) -> Result<Flow, Refused> {
    let menus = entry.menus;
    if menus.screens.entry.typing.is_set() {
        let failed = Callee::EditReplayName.run(entry)?;
        if !failed {
            if menus.screens.entry.typing.is_set() {
                return Ok(Flow::NextFrame);
            }
            if menus.tables.captions.left_by_key.is_set() {
                menus.tables.captions.left_by_key.set(false);
                menus.screens.draw_work.leave_result.set(0xFFFFFFFFu32);
                return Ok(Flow::Leave);
            }
            // 0x14 replays a page; the items count from 1.
            let replay = menus.tables.pages.replay_page.get().wrapping_mul(0x14);
            menus.screens.draw_work.leave_result.set(
                replay
                    .wrapping_add(menus.screens.loop_flags.item.get())
                    .wrapping_sub(1),
            );
            return Ok(Flow::Leave);
        }
    }
    Ok(Flow::On)
}

/// What the loop returns: `DrawWork.leave_result`, which the step above leaves at -1 or at the
/// replay's number.
fn result(menus: &Menus) -> u32 {
    menus.screens.draw_work.leave_result.get()
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
            table: |_| DRAW_LISTS.save_replay_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.save_replay_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.save_replay_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(ScreenFlag::EscapePressed, false)],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().save_replay_picture
    }))],
    result: Some(result),
    keys: Keys {
        escape_leaves: true,
        ..Keys::DEFAULT
    },
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |_| DRAW_LISTS.save_replay_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.save_replay_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.save_replay_overlay.id(),
                run: false,
            },
        ]),
        each_frame: &[Step::DoFlow(each_frame)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.save_replay.id(),
        slot::SAVE_REPLAY,
    )
};
