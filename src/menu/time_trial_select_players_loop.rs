//! `menu::time_trial_select_players_loop` (0x48BCDC): the loop of the screen where the number of players
//! is chosen.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `arcade1.png`; leaving is not asked about; the
//! steps that are more than a call or a store are the functions below, generated from the
//! listing.
//! Screen: the player count of the time trial, "ESCOLHA NÚMERO DE JOGADORES" (the owner
//! said so after Rust-run visits, 2026-10-11). Named `select_players_loop` until then.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::rva;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::menus_store::MENUS;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{
    Callee, Carried, List, Screen, Step, carousel, prepare_scene, screen_entry,
};
use crate::menu::state_store::MENU_STATE;
use crate::scene::draw_store::DRAW;

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        DRAW.view_offset[0].set(0x7u32);
        DRAW.view_offset[1].set(0xAu32);
        DRAW.view_zoom.set(0x100u32);
        prepare_scene(c);
        SCREEN_FLOW.turning.set(0x1u32);
        MENU_STATE.carousel.menu.set(0x2u32);
        carousel(c)?;
        c.arg = SCREEN_ITEMS.current[slot::TIME_TRIAL_SELECT_PLAYERS].get();
        LOOP_FLAGS.item.set(c.arg);
        c.arg = c.arg.wrapping_sub(1);
        MENU_STATE.carousel.front.set(c.arg);
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.time_trial_select_players.addr(),
    saved_item: || &SCREEN_ITEMS.current[slot::TIME_TRIAL_SELECT_PLAYERS],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.time_trial_select_players_scene.addr(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.time_trial_select_players_draw.addr(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.time_trial_select_players_overlay.addr(),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Write(|| &EVENT_STATE.event_done, 0x0)],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::ArgAddr(|| {
            crate::menu::texts_store::MENU_TEXTS
                .time_trial_select_players_picture
                .addr()
        }),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
    ],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Write(|| &OPTION_WORK.menu_sound, 0x6F)],
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
    screen_entry("menu::time_trial_select_players_loop", &SCREEN)
}
