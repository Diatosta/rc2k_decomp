//! `menu::audio_setup_loop` (0x4767E3): the loop of the audio set-up screen.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `audio.png`; leaving is not asked about; its
//! own 3D list goes to `g_sceneList` before the first frame only; the steps that are more than
//! a call or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "CONFIGURAÇÃO ÁUDIO", not yet a
//! visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Entry, Frames, List, Screen, Step, screen_entry};

fn before(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.items.current[slot::AUDIO_SETUP].set(screens.items.first[slot::AUDIO_SETUP].get());
    Ok(())
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
            table: |m| m.tables.draw_lists.audio_setup_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.audio_setup_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Do(before)],
    set_up_first: &[Step::Write(|m| &m.screens.flow.quiet, 0x1)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.audio_setup_picture
    }))],
    after: &[Step::Write(|m| &m.screens.flow.quiet, 0x0)],
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list, // g_sceneList
                table: |m| m.tables.draw_lists.audio_setup_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list, // g_drawList
                table: |m| m.tables.draw_lists.audio_setup_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
                table: |m| m.tables.draw_lists.audio_setup_overlay.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.audio_setup.id(),
        |m| &m.screens.items.current[slot::AUDIO_SETUP],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::audio_setup_loop", &SCREEN)
}
