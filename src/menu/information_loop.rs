//! The loop of the information screen. Screen-specific part of `menu::screen_loop`: background
//! `info.png`; leaving is not asked about.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_id::ScreenId;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, List, Screen, ScreenVar, SetUp, Step, log_line, screen_entry,
};

fn set_up_first(set_up: &SetUp) -> Result<(), Refused> {
    let screens = &set_up.menus.screens;
    log_line(c"InfoScr: Start");
    set_up.set_view(-156, 39, 0x140);
    set_up.prepare_scene();
    set_up.set_clip_sides(-40, 760);
    screens.flow.write().turning = 0x1u32;
    screens.carousel.write().menu = 0x9u32;
    set_up.carousel_with(None)?;
    let item = screens.items.current(slot::INFORMATION);
    screens.loop_flags.write().item = item;
    screens.carousel.write().front = item.wrapping_sub(1);
    Ok(())
}

fn after() -> Result<(), Refused> {
    log_line(c"InfoScr: End");
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.information_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.information_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.information_overlay.id(),
            run: false,
        },
    ],
    set_up_first: &[Step::DoSetUp(set_up_first)],
    background: &[
        Step::Write(ScreenVar::ClearsEscapeFlag, 0x1),
        Step::Call(Callee::LoadBackground(|m| &m.texts.information_picture)),
    ],
    after: &[Step::DoPlain(after)],
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.information.id(),
        slot::INFORMATION,
    )
};

pub fn run(o: crate::owners::Owners<'static>) -> Result<(), crate::errors::Refused> {
    screen_entry(o, ScreenId::Information)
}
