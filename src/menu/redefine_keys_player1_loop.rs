//! `menu::redefine_keys_player1_loop` (0x4796C4): the keyboard assignment screen "REDEFINIR
//! TECLADO" of the first car.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `defkeys.png`; leaving is not asked about; its
//! own 3D list goes to `g_sceneList` before the first frame only; it has a step of its own once
//! a frame; the steps that are more than a call or a store are the functions below, generated
//! from the listing.
//! The name is a reading from the title string of its menu, "REDEFINIR TECLADO", not yet a
//! visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, Frames, List, Screen, Step, quitting, screen_entry,
};

fn before(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    // The player's twelve bytes at +0x1C copied to `keys.redefined_keys`.
    {
        let keys = menus.play.players.players[0].get().keys;
        for (n, key) in keys.into_iter().enumerate() {
            menus.keys.tables.redefined_keys.set(n, key);
        }
        menus.screens.event.current_player.set(0x0u32);
    }
    Ok(())
}

fn after_first_frame(_: &Entry) -> Result<Flow, Refused> {
    if quitting() as u32 == 0x1 {
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

fn each_frame(entry: &Entry) -> Result<Flow, Refused> {
    let screens = &entry.menus.screens;
    if screens.option_work.redefining_key.is_set() {
        return Ok(Flow::NextFrame);
    }
    Ok(Flow::On)
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
            table: |m| m.tables.draw_lists.redefine_keys_player1_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.redefine_keys_player1_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Do(before)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.redefine_keys_player1_picture
    }))],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list, // g_sceneList
                table: |m| m.tables.draw_lists.redefine_keys_player1_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list, // g_drawList
                table: |m| m.tables.draw_lists.redefine_keys_player1_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
                table: |m| m.tables.draw_lists.redefine_keys_player1_overlay.id(),
                run: false,
            },
        ]),
        each_frame: &[Step::DoFlow(each_frame)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.redefine_keys_player1.id(),
        |m| &m.screens.items.current[slot::REDEFINE_KEYS_PLAYER1],
    )
};

/// Runs the screen (a refusal is the caller's).
pub fn run(o: crate::owners::Owners<'_>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, "menu::redefine_keys_player1_loop", &SCREEN)
}
