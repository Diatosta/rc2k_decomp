//! `menu::load_championship_46e899_loop` (0x46E899): a second screen titled "CARREGAR
//! CAMPEONATO", reached through a table.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is not asked about; on leaving the highlight is
//! saved in `dword_747678`, not in the variable it is taken from (the original's slip); the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "CARREGAR CAMPEONATO", not yet a
//! visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Entry, FrameRoutine, List, Screen, Step, prepare_scene, screen_entry,
};
use crate::scene::owner::scene;

// ---- Rust version: `menu::screen_loop` with this ----

fn prepare(_: &Entry) -> Result<(), Refused> {
    let drawing = &scene().drawing;
    drawing.scene.view_offset[0].set(0x0u32);
    drawing.scene.view_offset[1].set(0x0u32);
    drawing.scene.view_zoom.set(0x100u32);
    prepare_scene();
    drawing.scene.clip_edges[2].set(0x0u32);
    drawing.scene.clip_edges[3].set(0x320u32);
    drawing.scene.clip_screen[2].set(drawing.scene.clip_edges[2].bits() as i32 as f32);
    drawing.scene.clip_screen[3].set(drawing.scene.clip_edges[3].bits() as i32 as f32);
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || menus().tables.menus.load_championship_46e899.id(),
    saved_item: || &menus().screens.items.current[slot::LOAD_CHAMPIONSHIP_46E899],
    lists: &[
        List {
            pointer: || &menus().screens.flow.scene_list, // g_sceneList
            table: || {
                menus()
                    .tables
                    .draw_lists
                    .load_championship_46e899_scene
                    .id()
            },
            run: false,
        },
        List {
            pointer: || &menus().screens.flow.draw_list, // g_drawList
            table: || menus().tables.draw_lists.load_championship_46e899_draw.id(),
            run: false,
        },
        List {
            pointer: || &menus().screens.flow.overlay_list, // g_overlayList
            table: || {
                menus()
                    .tables
                    .draw_lists
                    .load_championship_46e899_overlay
                    .id()
            },
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Write(|| &menus().screens.flow.left_service, 0x0)],
    prepare: &[Step::Do(prepare)],
    after_lists: &[],
    background: &[],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Write(|| &menus().screens.flow.left_service, 0x1)],
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
    leave_saved_item: Some(|| &menus().screens.items.current[slot::SAVE_SETUP]),
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
    screen_entry("menu::load_championship_46e899_loop", &SCREEN)
}
