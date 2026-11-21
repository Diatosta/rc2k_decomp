//! The menu after the results screens, by its title "REVIEW OPTIONS". Screen-specific part of
//! `menu::screen_loop`: background `roptions.png`; leaving is asked about; also when
//! `loop_flags.ask_leave` is set.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, List, MenuSound, Screen, ScreenFlag, SetUp, Step};

fn set_up_first(set_up: &SetUp) -> Result<(), Refused> {
    let screens = &set_up.menus.screens;
    screens.loop_flags.write().ask_leave = false;
    set_up.set_view(-156, 39, 0x140);
    set_up.prepare_scene();
    screens.flow.write().turning = 0x1u32;
    screens.carousel.write().menu = 0xFu32;
    set_up.carousel_with(None)?;
    let item = screens.items.current(slot::CHAMPIONSHIP_SERVICE_REVIEW);
    screens.loop_flags.write().item = item;
    screens.carousel.write().front = item.wrapping_sub(1);
    screens.flow.write().turning = 0x0u32;
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.arcade_review_options_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.arcade_review_options_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.arcade_review_options_overlay.id(),
            run: false,
        },
    ],
    before: &[
        Step::WriteFlag(ScreenFlag::EventLeft, false),
        Step::WriteFlag(ScreenFlag::EventOver, false),
    ],
    set_up_first: &[Step::DoSetUp(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.arcade_review_options_picture
    }))],
    after: &[Step::Sound(MenuSound::None)],
    asking: Some(Asking {
        also_if: Some(ScreenFlag::AskLeave),
        ..Asking::DEFAULT
    }),
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.arcade_review_options.id(),
        slot::ARCADE_REVIEW_OPTIONS,
    )
};
