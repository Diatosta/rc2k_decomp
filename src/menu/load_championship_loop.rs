//! The loop of the screen where a saved championship is loaded. Screen-specific part of
//! `menu::screen_loop`: background `loadsv.png`; leaving is not asked about; its own 3D scene list
//! is set before the first frame only; the Escape flag is looked at at the top of every frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Screens;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Frames, Keys, List, Screen, ScreenFlag, Step, screen_entry,
};

fn before(screens: &Screens) -> Result<(), Refused> {
    screens.draw_work.escape_pressed.set(false);
    screens.items.current[slot::LOAD_CHAMPIONSHIP]
        .set(screens.items.first[slot::LOAD_CHAMPIONSHIP].get());
    Ok(())
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
            table: |_| DRAW_LISTS.load_championship_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.setup_list,
            table: |_| DRAW_LISTS.load_championship_setup.id(),
            run: true,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.load_championship_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::DoScreens(before)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.load_championship_picture
    }))],
    after: &[Step::WriteFlag(ScreenFlag::LeftByKey, false)],
    keys: Keys {
        escape_leaves: true,
        ..Keys::DEFAULT
    },
    frames: Frames {
        first_frame_lists: Some(&[
            List {
                pointer: |m| &m.screens.flow.scene_list,
                table: |_| DRAW_LISTS.load_championship_scene.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.draw_list,
                table: |_| DRAW_LISTS.load_championship_draw.id(),
                run: false,
            },
            List {
                pointer: |m| &m.screens.flow.overlay_list,
                table: |_| DRAW_LISTS.load_championship_overlay.id(),
                run: false,
            },
        ]),
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.load_championship.id(),
        slot::LOAD_CHAMPIONSHIP,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::LoadChampionship)
}
