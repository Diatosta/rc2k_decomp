//! The loop of the player details screen. Screen-specific part of `menu::screen_loop`: background
//! `pdetc.png`; leaving is not asked about; after an action it is left or set up again, nothing
//! else.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    ActionCheck, Callee, Entry, Keys, List, Screen, Step, carousel, prepare_scene, screen_entry,
};
use crate::menu::texts_store::texts;

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    entry.set_view(-156, 41, 0x140);
    prepare_scene(entry);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x8u32);
    carousel(entry, None)?;
    let item = screens.items.current[slot::PLAYER_DETAILS].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.player_details_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.player_details_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.player_details_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().player_details_picture
    }))],
    keys: Keys {
        action_check: ActionCheck::Short,
        ..Keys::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.player_details.id(),
        |m| &m.screens.items.current[slot::PLAYER_DETAILS],
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, &SCREEN)
}
