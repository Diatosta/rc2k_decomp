//! The loop of the options menu opened during an event. Screen-specific part of
//! `menu::screen_loop`: background `opts.png`; leaving is not asked about; the Escape flag is
//! looked at at the top of every frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Keys, List, Screen, ScreenFlag, Step, carousel, prepare_scene, screen_entry,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    entry.set_view(-156, 39, 0x140);
    prepare_scene(&entry.owners);
    screens.flow.turning.set(0x1u32);
    screens.carousel.menu.set(0x11u32);
    carousel(entry.menus, None)?;
    let item = screens.items.current[slot::IN_GAME_OPTIONS].get();
    screens.loop_flags.item.set(item);
    screens.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.in_game_options_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.in_game_options_list.from(2),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.in_game_options_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(ScreenFlag::EscapePressed, false)],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.in_game_options_picture
    }))],
    keys: Keys {
        escape_leaves: true,
        ..Keys::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.in_game_options.id(),
        slot::IN_GAME_OPTIONS,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::InGameOptions)
}
