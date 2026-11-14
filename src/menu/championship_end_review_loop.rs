//! The menu after the results screens, by its title "OPÇÕES DE REVISÃO". Screen-specific part of
//! `menu::screen_loop`: background `roptions.png`; leaving is asked about; also when
//! `loop_flags.ask_leave` is set; the Escape flag is looked at at the top of every frame; around an
//! action the highlight is kept in the saved highlight; the action run is that of the item of
//! another menu (a 1999 slip kept as is).

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Asking, Callee, Entry, Keys, List, Screen, Step, carousel, prepare_scene,
};
use crate::menu::texts_store::texts;

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    Callee::NoOp.run(entry)?;
    screens.loop_flags.ask_leave.set(false);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x14u32);
    carousel(entry, None)?;
    let item = screens.items.current[slot::CHAMPIONSHIP_END_REVIEW].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    entry.set_view(-156, 39, 0x140);
    prepare_scene(entry);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.championship_end_review_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.championship_end_review_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.championship_end_review_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::WriteFlag(
        |m| &m.screens.draw_work.escape_pressed,
        false,
    )],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|_| {
        &texts().championship_end_review_picture
    }))],
    after: &[Step::Call(Callee::NoOp)],
    asking: Some(Asking {
        also_if: Some(|m| &m.screens.loop_flags.ask_leave),
        confirmed: &[
            Step::WriteFlag(|m| &m.screens.loop_flags.ask_leave, false),
            Step::WriteFlag(|m| &m.tables.captions.left_by_key, false),
        ],
        ..Asking::DEFAULT
    }),
    keys: Keys {
        escape_leaves: true,
        action_saved_item: Some(|m| &m.screens.items.current[slot::CHAMPIONSHIP_RACES_REVIEW]),
        action_menu: Some(|m| m.tables.menus.championship_races_review.id()),
        ..Keys::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.championship_end_review.id(),
        |m| &m.screens.items.current[slot::CHAMPIONSHIP_END_REVIEW],
    )
};
