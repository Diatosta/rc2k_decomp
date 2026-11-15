//! The menu after the results screens, by its title "OPÇÕES DE REVISÃO". Screen-specific part of
//! `menu::screen_loop`: background `roptions.png`; leaving is asked about; also when
//! `loop_flags.ask_leave` is set; the next frame begins with the steps after the first frame.

use crate::errors::Refused;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Asking, Callee, Entry, Frames, List, MenuSound, Screen, ScreenFlag, ScreenVar, Step, carousel,
    prepare_scene,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.loop_flags.ask_leave.set(false);
    entry.set_view(-156, 39, 0x140);
    prepare_scene(&entry.owners);
    screens.flow.turning.set(0x1u32);
    screens.carousel.menu.set(0xFu32);
    carousel(entry, None)?;
    let item = screens.items.current[slot::SINGLE_RALLY_REVIEW].get();
    screens.loop_flags.item.set(item);
    screens.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list,
            table: |_| DRAW_LISTS.single_rally_review_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list,
            table: |_| DRAW_LISTS.single_rally_review_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list,
            table: |_| DRAW_LISTS.single_rally_review_overlay.id(),
            run: false,
        },
    ],
    before: &[
        Step::WriteFlag(ScreenFlag::EventLeft, false),
        Step::WriteFlag(ScreenFlag::EventOver, false),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.texts.single_rally_review_picture
    }))],
    after_first_frame: &[Step::Write(ScreenVar::ActionMode, 0x0)],
    after: &[Step::Sound(MenuSound::None)],
    asking: Some(Asking {
        also_if: Some(ScreenFlag::AskLeave),
        ..Asking::DEFAULT
    }),
    frames: Frames {
        next_frame_repeats_first: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        module_path!(),
        |m| m.tables.menus.single_rally_review.id(),
        slot::SINGLE_RALLY_REVIEW,
    )
};
