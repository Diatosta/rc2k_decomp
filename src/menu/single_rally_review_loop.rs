//! `menu::single_rally_review_loop` (0x485F5E): the menu after the results screens, by its
//! title "OPÇÕES DE REVISÃO" (named from its title string and callers; the original's name is lost)
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `roptions.png`; leaving is asked about; also
//! when `dword_755569` is 1; the next frame begins with the steps after the first frame; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "OPÇÕES DE REVISÃO", not yet a
//! visit.
//! Rust callers run its `entry`
//! (`routes.rs`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Asking, Callee, Entry, Frames, List, Screen, Step, carousel, prepare_scene,
};

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    screens.loop_flags.ended_a.set(0x0u32);
    crate::scene::polygon_work::front_end().window.view_offset[0].set(0xFFFFFF64u32);
    crate::scene::polygon_work::front_end().window.view_offset[1].set(0x27u32);
    crate::scene::polygon_work::front_end()
        .window
        .view_zoom
        .set(0x140u32);
    prepare_scene(entry.scene);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0xFu32);
    carousel(None)?;
    let item = screens.items.current[slot::SINGLE_RALLY_REVIEW].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.single_rally_review_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.single_rally_review_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.single_rally_review_overlay.id(),
            run: false,
        },
    ],
    before: &[
        Step::Write(|m| &m.screens.loop_flags.went_back, 0x0),
        Step::Write(|m| &m.screens.event.event_done, 0x0),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.single_rally_review_picture
    }))],
    after_first_frame: &[Step::Write(|m| &m.screens.draw_work.action_value, 0x0)],
    after: &[Step::Write(|m| &m.screens.option_work.menu_sound, 0x6F)],
    asking: Some(Asking {
        also_if: Some(|m| &m.screens.loop_flags.ended_a),
        ..Asking::DEFAULT
    }),
    frames: Frames {
        next_frame_repeats_first: true,
        ..Frames::DEFAULT
    },
    ..Screen::new(
        |m| m.tables.menus.single_rally_review.id(),
        |m| &m.screens.items.current[slot::SINGLE_RALLY_REVIEW],
    )
};
