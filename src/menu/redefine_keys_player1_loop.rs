//! The keyboard assignment screen "REDEFINIR TECLADO" of the first car. Screen-specific part of
//! `menu::screen_loop`: background `defkeys.png`; leaving is not asked about; its own 3D scene list
//! is set before the first frame only; it has a step of its own once a frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, Frames, List, Screen, Step, quitting, screen_entry,
};
use crate::menu::texts_store::texts;

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

fn after_first_frame(entry: &Entry) -> Result<Flow, Refused> {
    if quitting(entry.system) as u32 == 0x1 {
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
            pointer: |m| &m.screens.flow.unread_list,
            table: |_| DRAW_LISTS.unread_draw_list.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.redefine_keys_player1_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.redefine_keys_player1_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Do(before)],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().redefine_keys_player1_picture
    }))],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |_| DRAW_LISTS.redefine_keys_player1_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.redefine_keys_player1_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.redefine_keys_player1_overlay.id(),
                run: false,
            },
        ]),
        each_frame: &[Step::DoFlow(each_frame)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.redefine_keys_player1.id(),
        |m| &m.screens.items.current[slot::REDEFINE_KEYS_PLAYER1],
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, &SCREEN)
}
