//! The menu after the results screens, by its title "OPÇÕES DE REVISÃO". Screen-specific part of
//! `menu::screen_loop`: background `roptions.png`; leaving is asked about; also when
//! `loop_flags.ask_leave` is set.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Asking, Callee, Entry, List, MenuSound, Screen, Step, carousel, prepare_scene,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.loop_flags.ask_leave.set(false);
    entry.set_view(-156, 39, 0x140);
    prepare_scene(entry);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0xFu32);
    carousel(entry, None)?;
    let item = screens.items.current[slot::CHAMPIONSHIP_SERVICE_REVIEW].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |m| m.tables.draw_lists.arcade_review_options_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |m| m.tables.draw_lists.arcade_review_options_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |m| m.tables.draw_lists.arcade_review_options_overlay.id(),
            run: false,
        },
    ],
    before: &[
        Step::WriteFlag(|m| &m.screens.loop_flags.event_left, false),
        Step::WriteFlag(|m| &m.screens.event.event_over, false),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.arcade_review_options_picture
    }))],
    after: &[Step::Sound(MenuSound::None)],
    asking: Some(Asking {
        also_if: Some(|m| &m.screens.loop_flags.ask_leave),
        ..Asking::DEFAULT
    }),
    ..Screen::new(
        |m| m.tables.menus.arcade_review_options.id(),
        |m| &m.screens.items.current[slot::ARCADE_REVIEW_OPTIONS],
    )
};
