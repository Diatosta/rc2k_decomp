//! `menu::save_championship_loop` (0x47F034): the loop of the screen where a championship is
//! saved and its name typed.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `loadsv.png`; leaving is not asked about; its
//! own 3D list goes to `g_sceneList` before the first frame only; the Escape flag is looked at
//! at the top of every frame; it has a step of its own once a frame; the steps that are more
//! than a call or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "GRAVAR CAMPEONATO", not yet a
//! visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::rva;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::menus_store::MENUS;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{Callee, Carried, Flow, List, Screen, Step, screen_entry};

// ---- Rust version: `menu::screen_loop` with this ----

fn before(c: &mut Carried) -> Result<(), Refused> {
    DRAW_WORK.escaped.set(0x0u32);
    c.measure_x = SCREEN_ITEMS.first[slot::SAVE_CHAMPIONSHIP].get();
    SCREEN_ITEMS.current[slot::SAVE_CHAMPIONSHIP].set(c.measure_x);
    Ok(())
}

fn each_frame(c: &mut Carried) -> Result<Flow, Refused> {
    if OPTION_WORK.typing.get() == 0x1 {
        Callee::EditSavegameName.run(c)?;
        if OPTION_WORK.typing.get() != 0x0 {
            return Ok(Flow::NextFrame);
        }
        c.arg = 0x0;
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.save_championship.id(),
    saved_item: || &SCREEN_ITEMS.current[slot::SAVE_CHAMPIONSHIP],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.unread_list, // g_unreadList
            table: || DRAW_LISTS.unread_758b72.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.draw_758bbe.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.setup_list, // dword_755500
            table: || DRAW_LISTS.save_championship_setup.id(),
            run: true,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.overlay_75914a.id(),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.save_championship_scene.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.draw_758bbe.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.overlay_75914a.id(),
            run: false,
        },
    ]),
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgText(|| &crate::menu::texts_store::MENU_TEXTS.save_championship_picture),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
    ],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
    escape_leaves: true,
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
    screen_entry("menu::save_championship_loop", &SCREEN)
}
