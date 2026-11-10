//! `menu::review_options_481429_loop` (0x481429): the menu after the results screens, by its
//! title "OPÇÕES DE REVISÃO" (a reading)
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `roptions.png`; leaving is asked about; also
//! when `dword_755569` is 1; the Escape flag is looked at at the top of every frame; around an
//! action the highlight is kept in `dword_7477A4`; the action run is that of the item of
//! another menu, 0x754C88 (the original's slip); the steps that are more than a call or a store
//! are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "OPÇÕES DE REVISÃO", not yet a
//! visit.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, FrameRoutine, List, Screen, Step, carousel, prepare_scene,
};

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(entry: &Entry) -> Result<(), Refused> {
    let screens = &entry.menus.screens;
    Callee::NoOp.run(entry)?;
    screens.loop_flags.ended_a.set(0x0u32);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x14u32);
    carousel(None)?;
    let item = screens.items.current[slot::REVIEW_OPTIONS_481429].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    screens.flow.turning.set(0x0u32);
    crate::scene::polygon_work::front_end().window.view_offset[0].set(0xFFFFFF64u32);
    crate::scene::polygon_work::front_end().window.view_offset[1].set(0x27u32);
    crate::scene::polygon_work::front_end()
        .window
        .view_zoom
        .set(0x140u32);
    prepare_scene(entry.scene);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: |m| m.tables.menus.review_options_481429.id(),
    saved_item: |m| &m.screens.items.current[slot::REVIEW_OPTIONS_481429],
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.review_options_481429_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.review_options_481429_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.review_options_481429_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[Step::Write(|m| &m.screens.draw_work.escaped, 0x0)],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.review_options_481429_picture
    }))],
    after_first_frame: &[],
    confirm: Some(&[]),
    ask_also_if: Some(|m| &m.screens.loop_flags.ended_a),
    confirmed: &[
        Step::Write(|m| &m.screens.loop_flags.ended_a, 0x0),
        Step::Write(|m| &m.tables.captions.screen_left, 0x0),
    ],
    unasked_skips_save: false,
    after: &[Step::Call(Callee::NoOp)],
    escape_leaves: true,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: false,
    action_saved_item: Some(|m| &m.screens.items.current[slot::REVIEW_OPTIONS_48108E]),
    action_menu: Some(|m| m.tables.menus.review_options_48108e.id()),
    no_quit_check: false,
    before_asking: &[],
    ask_unless: None,
    leaving: &[],
    leave_saved_item: None,
    keys_menu: None,
    after_restored: &[],
    result: None,
    late_highlight: false,
    before_first_frame: &[],
    before_frame: &[],
    after_blink: &[],
    restart_action_check: false,
    always_asks: false,
};
