//! `menu::championship_end_review_loop` (0x481429): the menu after the results screens, by its
//! title "OPÇÕES DE REVISÃO" (named from its title string and callers; the original's name is lost)
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `roptions.png`; leaving is asked about; also
//! when `loop_flags.ask_leave` is 1; the Escape flag is looked at at the top of every frame; around an
//! action the highlight is kept in the saved highlight; the action run is that of the item of
//! another menu, 0x754C88 (the original's slip); the steps that are more than a call or a store
//! are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "OPÇÕES DE REVISÃO", not yet a
//! visit.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Asking, Callee, Entry, Keys, List, Screen, Step, carousel, prepare_scene,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    Callee::NoOp.run(entry)?;
    screens.loop_flags.ask_leave.set(0x0u32);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x14u32);
    carousel(None)?;
    let item = screens.items.current[slot::CHAMPIONSHIP_END_REVIEW].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    entry.set_view(-156, 39, 0x140);
    prepare_scene(entry.scene);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.championship_end_review_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.championship_end_review_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.championship_end_review_overlay.id(),
            run: false,
        },
    ],
    before: &[Step::Write(|m| &m.screens.draw_work.escape_pressed, 0x0)],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.championship_end_review_picture
    }))],
    after: &[Step::Call(Callee::NoOp)],
    asking: Some(Asking {
        also_if: Some(|m| &m.screens.loop_flags.ask_leave),
        confirmed: &[
            Step::Write(|m| &m.screens.loop_flags.ask_leave, 0x0),
            Step::Write(|m| &m.tables.captions.left_by_key, 0x0),
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
        |m| m.tables.menus.championship_end_review.id(),
        |m| &m.screens.items.current[slot::CHAMPIONSHIP_END_REVIEW],
    )
};
