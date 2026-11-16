//! The player count of the simple mode, titled "SELECT NUMBER OF PLAYERS", the mode menu's
//! first entry. Screen-specific part of `menu::screen_loop`: background `arcade1.png`; leaving
//! is not asked about; the next frame begins with the steps after the first frame. The
//! background's file name says "arcade", but the arcade mode is another one and has no player
//! count.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Tables;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Flow, Frames, List, MenuSound, Screen, ScreenFlag, SetUp, Step, screen_entry,
};

fn set_up_first(set_up: &SetUp) -> Result<(), Refused> {
    let screens = &set_up.menus.screens;
    screens.flow.turning.set(0x1u32);
    screens.carousel.menu.set(0x2u32);
    set_up.carousel_with(None)?;
    let item = screens.items.current[slot::SIMPLE_SELECT_PLAYERS].get();
    screens.loop_flags.item.set(item);
    screens.carousel.front.set(item.wrapping_sub(1));
    set_up.set_view(7, 10, 0x100);
    set_up.prepare_scene();
    Ok(())
}

fn after_first_frame(tables: &Tables) -> Result<Flow, Refused> {
    if tables.captions.left_by_key.is_set() {
        tables.captions.left_by_key.set(false);
        return Ok(Flow::SetUp);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.simple_select_players_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.simple_select_players_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.simple_select_players_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(ScreenFlag::EventOver, false)],
    set_up_first: &[Step::DoSetUp(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.simple_select_players_picture
    }))],
    after_first_frame: &[Step::DoTablesFlow(after_first_frame)],
    after: &[Step::Sound(MenuSound::None)],
    frames: Frames {
        next_frame_repeats_first: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.simple_select_players.id(),
        slot::SIMPLE_SELECT_PLAYERS,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::SimpleSelectPlayers)
}
