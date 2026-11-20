//! The mode menu, titled "START RALLY". Screen-specific part of `menu::screen_loop`: background
//! `sral.png`; leaving is not asked about; the next frame begins with the steps after the first
//! frame; it has steps of its own first on leaving; the menu sound is cleared every frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::owner::Menus;
use crate::menu::owner::Tables;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Flow, Frames, List, Screen, ScreenVar, SetUp, Step, screen_entry,
};

fn set_up_first(set_up: &SetUp) -> Result<(), Refused> {
    let screens = &set_up.menus.screens;
    screens.flow.write().turning = 0x1u32;
    screens.carousel.menu.set(0x1u32);
    set_up.carousel_with(None)?;
    let item = screens.items.current[slot::START_RALLY].get();
    screens.loop_flags.write().item = item;
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

fn leaving(menus: &Menus) -> Result<(), Refused> {
    if menus.tables.captions.left_by_key.is_set() {
        crate::menu::go_back::go_back(menus);
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.start_rally_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.start_rally_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.start_rally_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::DoSetUp(set_up_first)],
    background: &[
        Step::Write(ScreenVar::ClearsEscapeFlag, 0x1),
        Step::Call(Callee::LoadBackground(|m| &m.texts.start_rally_picture)),
    ],
    after_first_frame: &[Step::DoTablesFlow(after_first_frame)],
    leaving: &[Step::DoMenus(leaving)],
    frames: Frames {
        next_frame_repeats_first: true,
        clears_menu_sound: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.start_rally.id(),
        slot::START_RALLY,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::StartRally)
}
