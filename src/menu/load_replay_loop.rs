//! `menu::load_replay_loop` (0x47F48B): the replay list "CARREGAR REPETIÇÃO"; returns the
//! replay chosen.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `loadsv.png`; leaving is not asked about; its
//! own 3D list goes to `g_sceneList` before the first frame only; the Escape flag is looked at
//! at the top of every frame; it returns a value in eax; the steps that are more than a call or
//! a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "CARREGAR REPETIÇÃO", not yet a
//! visit.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Entry, Frames, Keys, List, Screen, Step};

fn before(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.items.current[slot::LOAD_REPLAY].set(screens.items.first[slot::LOAD_REPLAY].get());
    screens.draw_work.escaped.set(0x0u32);
    Ok(())
}

/// What the loop returns, written by hand from the end of the listing: -1 if the highlight is on item 0x16 or `dword_758806` is 1, else the replay's
/// number, `dword_755248` * 20 + the highlighted item - 1.
fn result(menus: &Menus) -> u32 {
    let item: u32 = menus.screens.loop_flags.item.get();
    if item == 0x16 || menus.tables.captions.screen_left.get() == 1 {
        return 0xFFFF_FFFF;
    }
    menus
        .tables
        .pages
        .replay_page
        .get()
        .wrapping_mul(0x14)
        .wrapping_add(item)
        .wrapping_sub(1)
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
            table: |m| m.tables.draw_lists.draw_758bd6.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list, // dword_755500
            table: |m| m.tables.draw_lists.load_replay_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.overlay_759152.id(),
            run: false,
        },
    ],
    before: &[Step::Do(before)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.load_replay_picture
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
                table: |m| m.tables.draw_lists.load_replay_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list, // g_drawList
                table: |m| m.tables.draw_lists.draw_758bd6.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
                table: |m| m.tables.draw_lists.overlay_759152.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.load_replay.id(),
        |m| &m.screens.items.current[slot::LOAD_REPLAY],
    )
};
