//! `menu::save_setup_loop` (0x46EB7A): the loop of the screen where a car setup is saved and
//! its name typed.
//!
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is not asked about; it has a step of its own once
//! a frame; the steps that are more than a call or a store are the functions below, generated
//! from the listing.
//! The name is a reading from the title string of its menu, "{}", not yet a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, FrameRoutine, List, Screen, Step, prepare_scene, screen_entry,
};

// ---- Rust version: `menu::screen_loop` with this ----

fn prepare(entry: &Entry) -> Result<(), Refused> {
    let drawing = &entry.scene.drawing;
    drawing.scene.view_offset[0].set(0x0u32);
    drawing.scene.view_offset[1].set(0x0u32);
    drawing.scene.view_zoom.set(0x100u32);
    prepare_scene(entry.scene);
    drawing.scene.clip_edges[2].set(0x0u32);
    drawing.scene.clip_edges[3].set(0x320u32);
    drawing.scene.clip_screen[2].set(drawing.scene.clip_edges[2].bits() as i32 as f32);
    drawing.scene.clip_screen[3].set(drawing.scene.clip_edges[3].bits() as i32 as f32);
    Ok(())
}

fn each_frame(entry: &Entry) -> Result<Flow, Refused> {
    let screens = &entry.menus.screens;
    if screens.option_work.typing.get() == 0x1 {
        Callee::EditSetupName.run(entry)?;
        if screens.option_work.typing.get() != 0x0 {
            return Ok(Flow::NextFrame);
        }
        screens.draw_work.action_value.set(0x2u32);
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    menu: |m| m.tables.menus.save_setup.id(),
    saved_item: |m| &m.screens.items.current[slot::SAVE_SETUP],
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.save_setup_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.save_setup_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.save_setup_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Write(|m| &m.screens.flow.left_service, 0x0)],
    prepare: &[Step::Do(prepare)],
    after_lists: &[],
    background: &[],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Write(|m| &m.screens.flow.left_service, 0x1)],
    escape_leaves: false,
    each_frame: &[Step::DoFlow(each_frame)],
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
    screen_entry("menu::save_setup_loop", &SCREEN)
}
