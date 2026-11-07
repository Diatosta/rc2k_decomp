//! `menu::information_loop` (0x48E169): the loop of the information screen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `info.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! The name is a reading from the title string of its menu, "INFORMAÇÃO", not yet a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, FrameRoutine, List, Screen, Step, carousel, log_line, prepare_scene,
    screen_entry,
};
use crate::scene::owner::scene;

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(_: &Entry) -> Result<(), Refused> {
    let drawing = &scene().drawing;
    let screens = &menus().screens;
    // Between `pusha` and `popa`: no register changes.
    log_line(c"InfoScr: Start");
    drawing.scene.view_offset[0].set(0xFFFFFF64u32);
    drawing.scene.view_offset[1].set(0x27u32);
    drawing.scene.view_zoom.set(0x140u32);
    prepare_scene();
    drawing.scene.clip_edges[2].set(0xFFFFFFD8u32);
    drawing.scene.clip_edges[3].set(0x2F8u32);
    drawing.scene.clip_screen[2].set(drawing.scene.clip_edges[2].bits() as i32 as f32);
    drawing.scene.clip_screen[3].set(drawing.scene.clip_edges[3].bits() as i32 as f32);
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x9u32);
    carousel(None)?;
    let item = screens.items.current[slot::INFORMATION].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    Ok(())
}

fn after(_: &Entry) -> Result<(), Refused> {
    // Between `pusha` and `popa`: no register changes.
    log_line(c"InfoScr: End");
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: |m| m.tables.menus.information.id(),
    saved_item: |m| &m.screens.items.current[slot::INFORMATION],
    lists: &[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.information_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.information_draw.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.information_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::Write(|m| &m.screens.event.clears_escape_flag, 0x1),
        Step::Call(Callee::LoadBackground(|m| {
            &m.tables.texts.information_picture
        })),
    ],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Do(after)],
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
    screen_entry("menu::information_loop", &SCREEN)
}
