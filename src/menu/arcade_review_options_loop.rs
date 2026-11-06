//! `menu::arcade_review_options_loop` (0x488958): the menu after the results screens, by its
//! title "OPÇÕES DE REVISÃO" (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`), as
//! extracted from the disassembly: background `roptions.png`; leaving is asked about; also when
//! `dword_755569` is 1; the steps that are more than a call or a store are the functions below,
//! generated from the listing. Named by its address until then. visit.
//! Callers: `sub_487E2C`.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, FrameRoutine, List, Screen, Step, carousel, prepare_scene,
};
use crate::scene::draw_store::DRAW;

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(_: &Entry) -> Result<(), Refused> {
    crate::menu::owner::menus()
        .screens
        .loop_flags
        .ended_a
        .set(0x0u32);
    DRAW.view_offset[0].set(0xFFFFFF64u32);
    DRAW.view_offset[1].set(0x27u32);
    DRAW.view_zoom.set(0x140u32);
    prepare_scene();
    crate::menu::owner::menus().screens.flow.turning.set(0x1u32);
    crate::menu::owner::menus()
        .screens
        .state
        .carousel
        .menu
        .set(0xFu32);
    carousel(None)?;
    let item = crate::menu::owner::menus().screens.items.current[slot::REVIEW_OPTIONS_47D5A5].get();
    crate::menu::owner::menus()
        .screens
        .loop_flags
        .item
        .set(item);
    crate::menu::owner::menus()
        .screens
        .state
        .carousel
        .front
        .set(item.wrapping_sub(1));
    crate::menu::owner::menus().screens.flow.turning.set(0x0u32);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || {
        crate::menu::owner::menus()
            .tables
            .menus
            .arcade_review_options
            .id()
    },
    saved_item: || &crate::menu::owner::menus().screens.items.current[slot::ARCADE_REVIEW_OPTIONS],
    lists: &[
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.scene_list, // g_sceneList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .arcade_review_options_scene
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.draw_list, // g_drawList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .arcade_review_options_draw
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.overlay_list, // g_overlayList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .arcade_review_options_overlay
                    .id()
            },
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[
        Step::Write(
            || &crate::menu::owner::menus().screens.loop_flags.went_back,
            0x0,
        ),
        Step::Write(
            || &crate::menu::owner::menus().screens.event.event_done,
            0x0,
        ),
    ],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|| {
        &crate::menu::owner::menus()
            .tables
            .texts
            .arcade_review_options_picture
    }))],
    after_first_frame: &[],
    confirm: Some(&[]),
    ask_also_if: Some(|| &crate::menu::owner::menus().screens.loop_flags.ended_a),
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Write(
        || &crate::menu::owner::menus().screens.option_work.menu_sound,
        0x6F,
    )],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: false,
    action_saved_item: None,
    action_menu: None,
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
