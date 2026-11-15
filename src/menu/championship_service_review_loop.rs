//! The menu after the results screens, by its title "OPÇÕES DE REVISÃO". Screen-specific part of
//! `menu::screen_loop`: background `roptions.png`; leaving is asked about; also when
//! `loop_flags.ask_leave` is set; the Escape flag is looked at at the top of every frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Asking, Callee, Entry, Keys, List, Screen, ScreenFlag, ScreenVar, Step, carousel, prepare_scene,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    Callee::NoOp.run(entry)?;
    screens.loop_flags.ask_leave.set(false);
    screens.flow.turning.set(0x1u32);
    screens.carousel.menu.set(0xAu32);
    carousel(entry.menus, None)?;
    let item = screens.items.current[slot::CHAMPIONSHIP_SERVICE_REVIEW].get();
    screens.loop_flags.item.set(item);
    screens.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    entry.set_view(-156, 39, 0x140);
    prepare_scene(&entry.owners);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.championship_service_review_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.championship_service_review_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.championship_service_review_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(ScreenFlag::EscapePressed, false)],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.championship_service_review_picture
    }))],
    after: &[
        Step::Call(Callee::NoOp),
        Step::Write(ScreenVar::ServiceNext, 0x0),
    ],
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
        |m| m.tables.menus.championship_service_review.id(),
        slot::CHAMPIONSHIP_SERVICE_REVIEW,
    )
};
