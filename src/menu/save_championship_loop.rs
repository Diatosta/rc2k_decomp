//! `menu::save_championship_loop` (0x47F034): the loop of the screen where a championship is
//! saved and its name typed.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `loadsv.png`; leaving is not asked about; its
//! own 3D list goes to `g_sceneList` before the first frame only; the Escape flag is looked at
//! at the top of every frame; it has a step of its own once a frame; the steps that are more
//! than a call or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "GRAVAR CAMPEONATO", not yet a
//! visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, Frames, Keys, List, Screen, Step, screen_entry,
};

fn before(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.draw_work.escaped.set(0x0u32);
    screens.items.current[slot::SAVE_CHAMPIONSHIP]
        .set(screens.items.first[slot::SAVE_CHAMPIONSHIP].get());
    Ok(())
}

fn each_frame(entry: &Entry) -> Result<Flow, Refused> {
    let screens = &entry.menus.screens;
    if screens.option_work.typing.get() == 0x1 {
        Callee::EditSavegameName.run(entry)?;
        if screens.option_work.typing.get() != 0x0 {
            return Ok(Flow::NextFrame);
        }
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list, // g_unreadList
            table: |m| m.tables.draw_lists.unread_758b72.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.draw_758bbe.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list, // dword_755500
            table: |m| m.tables.draw_lists.save_championship_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.overlay_75914a.id(),
            run: false,
        },
    ],
    before: &[Step::Do(before)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.save_championship_picture
    }))],
    keys: Keys {
        escape_leaves: true,
        ..Keys::DEFAULT
    },
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list, // g_sceneList
                table: |m| m.tables.draw_lists.save_championship_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list, // g_drawList
                table: |m| m.tables.draw_lists.draw_758bbe.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
                table: |m| m.tables.draw_lists.overlay_75914a.id(),
                run: false,
            },
        ]),
        each_frame: &[Step::DoFlow(each_frame)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.save_championship.id(),
        |m| &m.screens.items.current[slot::SAVE_CHAMPIONSHIP],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::save_championship_loop", &SCREEN)
}
