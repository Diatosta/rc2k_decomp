//! `menu::view_best_times_loop` (0x48E494): the loop of the best times screen opened from the
//! menus.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `btimes.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "MELHORES TEMPOS", not yet a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{Callee, Entry, FrameRoutine, List, Screen, Step, screen_entry};

// ---- Rust version: `menu::screen_loop` with this ----

fn before(_: &Entry) -> Result<(), Refused> {
    crate::menu::owner::menus().screens.items.current[slot::VIEW_BEST_TIMES]
        .set(crate::menu::owner::menus().screens.items.first[slot::VIEW_BEST_TIMES].get());
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || {
        crate::menu::owner::menus()
            .tables
            .menus
            .view_best_times
            .id()
    },
    saved_item: || &crate::menu::owner::menus().screens.items.current[slot::VIEW_BEST_TIMES],
    lists: &[
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.scene_list, // g_sceneList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .view_best_times_scene
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
                    .view_best_times_draw
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.setup_list, // dword_755500
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .view_best_times_setup
                    .id()
            },
            run: true,
        },
        List {
            pointer: || &crate::menu::owner::menus().screens.flow.overlay_list, // g_overlayList
            table: || {
                crate::menu::owner::menus()
                    .tables
                    .draw_lists
                    .view_best_times_overlay
                    .id()
            },
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|| {
        &crate::menu::owner::menus()
            .tables
            .texts
            .view_best_times_picture
    }))],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
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

/// The Rust version where the typed tables run it (a refusal is the
/// caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::view_best_times_loop", &SCREEN)
}
