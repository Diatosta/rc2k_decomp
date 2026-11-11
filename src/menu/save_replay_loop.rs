//! `menu::save_replay_loop` (0x47F77B): the replay naming screen "GRAVAR REPETIÇÃO"; returns
//! the slot the replay goes to.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `loadsv.png`; leaving is not asked about; its
//! own 3D list goes to `g_sceneList` before the first frame only; the Escape flag is looked at
//! at the top of every frame; it has a step of its own once a frame; it returns a value in eax;
//! the steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "GRAVAR REPETIÇÃO", not yet a
//! visit.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Entry, Flow, Frames, Keys, List, Screen, Step};

fn each_frame(entry: &Entry) -> Result<Flow, Refused> {
    let menus = entry.menus;
    if menus.screens.option_work.typing.is_set() {
        let failed = Callee::EditReplayName.run(entry)?;
        if !failed {
            if menus.screens.option_work.typing.is_set() {
                return Ok(Flow::NextFrame);
            }
            if menus.tables.captions.left_by_key.is_set() {
                menus.tables.captions.left_by_key.set(false);
                menus.screens.draw_work.scratch_0c.set(0xFFFFFFFFu32);
                return Ok(Flow::Leave);
            }
            // 0x14 replays a page; the items count from 1.
            let replay = menus.tables.pages.replay_page.get().wrapping_mul(0x14);
            menus.screens.draw_work.scratch_0c.set(
                replay
                    .wrapping_add(menus.screens.loop_flags.item.get())
                    .wrapping_sub(1),
            );
            return Ok(Flow::Leave);
        }
    }
    Ok(Flow::On)
}

/// What the loop returns, written by hand from the end of the listing: `DrawWork.scratch_0c`, which the step above leaves at -1 or at the replay's number.
fn result(menus: &Menus) -> u32 {
    menus.screens.draw_work.scratch_0c.get()
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list, // g_unreadList
            table: |m| m.tables.draw_lists.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.save_replay_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |m| m.tables.draw_lists.save_replay_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.save_replay_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(
        |m| &m.screens.draw_work.escape_pressed,
        false,
    )],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.save_replay_picture
    }))],
    result: Some(result),
    keys: Keys {
        escape_leaves: true,
        ..Keys::DEFAULT
    },
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list, // g_sceneList
                table: |m| m.tables.draw_lists.save_replay_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list, // g_drawList
                table: |m| m.tables.draw_lists.save_replay_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
                table: |m| m.tables.draw_lists.save_replay_overlay.id(),
                run: false,
            },
        ]),
        each_frame: &[Step::DoFlow(each_frame)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.save_replay.id(),
        |m| &m.screens.items.current[slot::SAVE_REPLAY],
    )
};
