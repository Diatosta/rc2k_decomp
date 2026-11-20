//! The menu after the results screens, by its title "REVIEW OPTIONS". Screen-specific part of
//! `menu::screen_loop`: background `roptions.png`; leaving is asked about; also when
//! `loop_flags.ask_leave` is set; the Escape flag is looked at at the top of every frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Asking, Callee, Keys, List, Screen, ScreenFlag, SetUp, Step};

fn set_up_first(set_up: &SetUp) -> Result<(), Refused> {
    let screens = &set_up.menus.screens;
    screens.loop_flags.write().ask_leave = false;
    screens.flow.write().turning = 0x1u32;
    screens.carousel.menu.set(0xAu32);
    set_up.carousel_with(None)?;
    let item = screens.items.current[slot::CHAMPIONSHIP_STANDINGS_REVIEW].get();
    screens.loop_flags.write().item = item;
    screens.carousel.front.set(item.wrapping_sub(1));
    screens.flow.write().turning = 0x0u32;
    set_up.set_view(-156, 39, 0x140);
    set_up.prepare_scene();
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.championship_standings_review_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.championship_standings_review_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.championship_standings_review_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(ScreenFlag::EscapePressed, false)],
    set_up_first: &[Step::DoSetUp(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.championship_standings_review_picture
    }))],
    after: &[],
    asking: Some(Asking {
        also_if: Some(ScreenFlag::AskLeave),
        confirmed: &[
            Step::WriteFlag(ScreenFlag::AskLeave, false),
            Step::WriteFlag(ScreenFlag::LeftByKey, false),
        ],
        ..Asking::DEFAULT
    }),
    keys: Keys {
        escape_leaves: true,
        ..Keys::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.championship_standings_review.id(),
        slot::CHAMPIONSHIP_STANDINGS_REVIEW,
    )
};
