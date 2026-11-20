//! The loop of the player details screen. Screen-specific part of `menu::screen_loop`: background
//! `pdetc.png`; leaving is not asked about; after an action it is left or set up again, nothing
//! else.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    ActionCheck, Callee, Keys, List, Screen, SetUp, Step, screen_entry,
};

fn set_up_first(set_up: &SetUp) -> Result<(), Refused> {
    let screens = &set_up.menus.screens;
    set_up.set_view(-156, 41, 0x140);
    set_up.prepare_scene();
    screens.flow.write().turning = 0x1u32;
    screens.carousel.menu.set(0x8u32);
    set_up.carousel_with(None)?;
    let item = screens.items.current[slot::PLAYER_DETAILS].get();
    screens.loop_flags.write().item = item;
    screens.carousel.front.set(item.wrapping_sub(1));
    screens.flow.write().turning = 0x0u32;
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
    set_up_first: &[Step::DoSetUp(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.player_details_picture
    }))],
    keys: Keys {
        action_check: ActionCheck::Short,
        ..Keys::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.player_details.id(),
        slot::PLAYER_DETAILS,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::PlayerDetails)
}
