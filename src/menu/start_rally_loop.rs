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
use crate::framework::rva;
use crate::menu::captions_store::CAPTIONS;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::menus_store::MENUS;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{
    Callee, Carried, Flow, List, Screen, Step, carousel, prepare_scene, screen_entry,
};
use crate::menu::state_store::MENU_STATE;
use crate::scene::draw_store::DRAW;

// ---- Rust version: `menu::screen_loop` with this ----

fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    SCREEN_FLOW.turning.set(0x1u32);
    MENU_STATE.carousel.menu.set(0x1u32);
    carousel(c)?;
    c.arg = SCREEN_ITEMS.current[slot::START_RALLY].get();
    LOOP_FLAGS.item.set(c.arg);
    c.arg = c.arg.wrapping_sub(1);
    MENU_STATE.carousel.front.set(c.arg);
    DRAW.view_offset[0].set(0x7u32);
    DRAW.view_offset[1].set(0xAu32);
    DRAW.view_zoom.set(0x100u32);
    prepare_scene(c);
    Ok(())
}

fn after_first_frame(_c: &mut Carried) -> Result<Flow, Refused> {
    if CAPTIONS.screen_left.get() == 0x1 {
        CAPTIONS.screen_left.set(0x0u32);
        return Ok(Flow::SetUp);
    }
    Ok(Flow::On)
}

fn leaving(c: &mut Carried) -> Result<(), Refused> {
    if CAPTIONS.screen_left.get() == 0x1 {
        Callee::GoBack.run(c)?;
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.start_rally.id(),
    saved_item: || &SCREEN_ITEMS.current[slot::START_RALLY],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.start_rally_scene.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.start_rally_draw.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.start_rally_overlay.id(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::Write(|| &EVENT_STATE.clears_escape_flag, 0x1),
        Step::ArgText(|| &crate::menu::texts_store::MENU_TEXTS.start_rally_picture),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
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
