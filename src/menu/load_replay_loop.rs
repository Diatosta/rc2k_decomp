//! The replay list "LOAD REPLAY"; returns the replay chosen. Screen-specific part of
//! `menu::screen_loop`: background `loadsv.png`; leaving is not asked about; its own 3D scene list
//! is set before the first frame only; the Escape flag is looked at at the top of every frame; it
//! returns a value.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Frames, Keys, List, Screen, Step};

fn before(screens: &Screens) -> Result<(), Refused> {
    screens.items.current[slot::LOAD_REPLAY].set(screens.items.first[slot::LOAD_REPLAY].get());
    screens.draw_work.write().escape_pressed = false;
    Ok(())
}

/// What the loop returns, written by hand from the end of the listing: -1 if the highlight is on
/// item 0x16 or `CaptionTables.left_by_key` is 1, else the replay's number, `MenuPages.replay_page`
/// * 20 + the highlighted item - 1.
fn result(menus: &Menus) -> u32 {
    let item: u32 = menus.screens.loop_flags.item.get();
    if item == 0x16 || menus.tables.captions.left_by_key.is_set() {
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
            pointer: |m| &m.screens.flow.unread_list,
            table: |_| DRAW_LISTS.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.load_replay_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.load_replay_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.load_replay_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::DoScreens(before)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.load_replay_picture
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
                table: |_| DRAW_LISTS.load_replay_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.load_replay_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.load_replay_overlay.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.load_replay.id(),
        slot::LOAD_REPLAY,
    )
};
