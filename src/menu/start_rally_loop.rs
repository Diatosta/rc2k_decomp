//! `menu::start_rally_loop` (0x475C12): the mode menu, titled "COMEÇAR RALI".
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `sral.png`; leaving is not asked about; the
//! next frame begins with the steps after the first frame; it has steps of its own first on
//! leaving; the quit switch is not looked at; the steps that are more than a call or a store
//! are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "COMEÇAR RALI", not yet a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, FrameRoutine, List, Screen, Step, carousel, prepare_scene, screen_entry,
};
use crate::scene::owner::scene;

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(_: &Entry) -> Result<(), Refused> {
    let drawing = &scene().drawing;
    let screens = &menus().screens;
    screens.flow.turning.set(0x1u32);
    screens.state.carousel.menu.set(0x1u32);
    carousel(None)?;
    let item = screens.items.current[slot::START_RALLY].get();
    screens.loop_flags.item.set(item);
    screens.state.carousel.front.set(item.wrapping_sub(1));
    drawing.scene.view_offset[0].set(0x7u32);
    drawing.scene.view_offset[1].set(0xAu32);
    drawing.scene.view_zoom.set(0x100u32);
    prepare_scene();
    Ok(())
}

fn after_first_frame(_: &Entry) -> Result<Flow, Refused> {
    let tables = &menus().tables;
    if tables.captions.screen_left.get() == 0x1 {
        tables.captions.screen_left.set(0x0u32);
        return Ok(Flow::SetUp);
    }
    Ok(Flow::On)
}

fn leaving(_: &Entry) -> Result<(), Refused> {
    if menus().tables.captions.screen_left.get() == 0x1 {
        Callee::GoBack.run()?;
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || menus().tables.menus.start_rally.id(),
    saved_item: || &menus().screens.items.current[slot::START_RALLY],
    lists: &[
        List {
            pointer: || &menus().screens.flow.scene_list, // g_sceneList
            table: || menus().tables.draw_lists.start_rally_scene.id(),
            run: false,
        },
        List {
            pointer: || &menus().screens.flow.draw_list, // g_drawList
            table: || menus().tables.draw_lists.start_rally_draw.id(),
            run: false,
        },
        List {
            pointer: || &menus().screens.flow.overlay_list, // g_overlayList
            table: || menus().tables.draw_lists.start_rally_overlay.id(),
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
        Step::Write(|| &menus().screens.event.clears_escape_flag, 0x1),
        Step::Call(Callee::LoadBackground(|| {
            &menus().tables.texts.start_rally_picture
        })),
    ],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
    escape_leaves: false,
    each_frame: &[],
    short_action_check: false,
    leave_at_once: None,
    next_frame_repeats_first: true,
    action_saved_item: None,
    action_menu: None,
    no_quit_check: true,
    before_asking: &[],
    ask_unless: None,
    leaving: &[Step::Do(leaving)],
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
    screen_entry("menu::start_rally_loop", &SCREEN)
}
