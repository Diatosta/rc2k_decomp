//! The mode menu's third entry, by its title "SELECT LEVEL". Screen-specific part of
//! `menu::screen_loop`: background `Selarc.png`; leaving is not asked about; the next frame begins
//! with the steps after the first frame.

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
    set_up.set_view(7, 10, 0x100);
    set_up.prepare_scene();
    screens.flow.write().turning = 0x1u32;
    screens.carousel.write().menu = 0x5u32;
    set_up.carousel_with(None)?;
    let item = screens.items.current(slot::ARCADE_SELECT_LEVEL);
    screens.loop_flags.write().item = item;
    screens.carousel.write().front = item.wrapping_sub(1);
    Ok(())
}

fn after_first_frame(tables: &Tables) -> Result<Flow, Refused> {
    if tables.captions.read().left_by_key {
        tables.captions.write().left_by_key = false;
        return Ok(Flow::SetUp);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.arcade_select_level_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.arcade_select_level_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.arcade_select_level_overlay.id(),
            run: false,
        },
    ],
    before: &[
        Step::WriteFlag(ScreenFlag::LeftByKey, false),
        Step::WriteFlag(ScreenFlag::EventOver, false),
    ],
    set_up_first: &[Step::DoSetUp(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.arcade_select_level_picture
    }))],
    after_first_frame: &[Step::DoTablesFlow(after_first_frame)],
    after: &[Step::Sound(MenuSound::None)],
    frames: Frames {
        next_frame_repeats_first: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.arcade_select_level.id(),
        slot::ARCADE_SELECT_LEVEL,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::ArcadeSelectLevel)
}
