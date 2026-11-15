//! The network rally menu, by its title "RALI EM REDE". Screen-specific part of
//! `menu::screen_loop`: background `nral.png`; leaving is not asked about; the quit switch is not
//! looked at.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Frames, List, MenuSound, SceneVar, Screen, ScreenFlag, SetUp, Step, screen_entry,
};

fn set_up_first(set_up: &SetUp) -> Result<(), Refused> {
    let screens = &set_up.menus.screens;
    screens.flow.turning.set(0x1u32);
    screens.carousel.menu.set(0x6u32);
    set_up.carousel_with(None)?;
    let item = screens.items.current[slot::NETWORK_RALLY].get();
    screens.loop_flags.item.set(item);
    screens.carousel.front.set(item.wrapping_sub(1));

    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.network_rally_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.network_rally_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.network_rally_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::DoSetUp(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.network_rally_picture
    }))],
    after_first_frame: &[
        Step::WriteScene(SceneVar::ViewOffsetX, 0x7),
        Step::WriteScene(SceneVar::ViewOffsetY, 0xA),
        Step::WriteScene(SceneVar::ViewZoom, 0x100),
        Step::Call(Callee::PrepareScene),
    ],
    after: &[
        Step::WriteFlag(ScreenFlag::LeftByKey, false),
        Step::WriteFlag(ScreenFlag::AskLeave, false),
        Step::WriteFlag(ScreenFlag::EscapePressed, false),
        Step::Sound(MenuSound::None),
        Step::WriteFlag(ScreenFlag::LeaveScreen, false),
    ],
    frames: Frames {
        no_quit_check: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.network_rally.id(),
        slot::NETWORK_RALLY,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::NetworkRally)
}
