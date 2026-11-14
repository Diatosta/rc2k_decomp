//! The loop of the screen where a championship is saved and its name typed. Screen-specific part of
//! `menu::screen_loop`: background `loadsv.png`; leaving is not asked about; its own 3D scene list
//! is set before the first frame only; the Escape flag is looked at at the top of every frame; it
//! has a step of its own once a frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, Frames, Keys, List, Screen, Step, screen_entry,
};
use crate::menu::texts_store::texts;

fn before(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.draw_work.escape_pressed.set(false);
    screens.items.current[slot::SAVE_CHAMPIONSHIP]
        .set(screens.items.first[slot::SAVE_CHAMPIONSHIP].get());
    Ok(())
}

fn each_frame(entry: &Entry) -> Result<Flow, Refused> {
    let screens = &entry.menus.screens;
    if screens.option_work.typing.is_set() {
        Callee::EditSavegameName.run(entry)?;
        if screens.option_work.typing.is_set() {
            return Ok(Flow::NextFrame);
        }
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
            table: |_| DRAW_LISTS.save_championship_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.save_championship_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.save_championship_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Do(before)],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().save_championship_picture
    }))],
    keys: Keys {
        escape_leaves: true,
        ..Keys::DEFAULT
    },
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |_| DRAW_LISTS.save_championship_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.save_championship_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.save_championship_overlay.id(),
                run: false,
            },
        ]),
        each_frame: &[Step::DoFlow(each_frame)],
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.save_championship.id(),
        |m| &m.screens.items.current[slot::SAVE_CHAMPIONSHIP],
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, &SCREEN)
}
