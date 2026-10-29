//! `menu::redefine_keys_player1_loop` (0x4796C4): the keyboard assignment screen "REDEFINIR
//! TECLADO" of the first car.
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: background `defkeys.png`; leaving is not asked about; its
//! own 3D list goes to `g_sceneList` before the first frame only; it has a step of its own once
//! a frame; the steps that are more than a call or a store are the functions below, generated
//! from the listing.
//! The name is a reading from the title string of its menu, "REDEFINIR TECLADO", not yet a
//! visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::rva;
use crate::menu::draw_lists_store::DRAW_LISTS;
use crate::menu::event_store::EVENT_STATE;
use crate::menu::flow_store::SCREEN_FLOW;
use crate::menu::keys_store::KEY_TABLES;
use crate::menu::menus_store::MENUS;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::players_store::PLAYER_TABLE;
use crate::menu::screen_items_store::{SCREEN_ITEMS, slot};
use crate::menu::screen_loop::{Callee, Carried, Flow, List, Screen, Step, quitting, screen_entry};

// ---- Rust version: `menu::screen_loop` with this ----

fn before(_c: &mut Carried) -> Result<(), Refused> {
    // The player's twelve bytes at +0x1C copied to `dword_755595` (`rep movsb` between
    // `pusha` and `popa`; eax pushed and popped around the player's address: no carried
    // value changes).
    {
        let player = PLAYER_TABLE.pointers[0].get();
        let keys = crate::pointers::GamePtr::new(player.wrapping_add(0x1C))
            .read_bytes(0xC, "a player's keys")?;
        for (n, key) in keys.into_iter().enumerate() {
            KEY_TABLES.redefined_keys.set(n, key);
        }
        EVENT_STATE.current_player.set(0x0u32);
    }
    Ok(())
}

fn after_first_frame(_c: &mut Carried) -> Result<Flow, Refused> {
    if quitting() as u32 == 0x1 {
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

fn each_frame(_c: &mut Carried) -> Result<Flow, Refused> {
    if OPTION_WORK.redefining_key.get() == 0x1 && OPTION_WORK.redefining_key.get() != 0x0 {
        return Ok(Flow::NextFrame);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    menu: || MENUS.redefine_keys_player1.id(),
    saved_item: || &SCREEN_ITEMS.current[slot::REDEFINE_KEYS_PLAYER1],
    lists: &[
        List {
            pointer: || &SCREEN_FLOW.unread_list, // g_unreadList
            table: || DRAW_LISTS.unread_758b72.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.draw_758de2.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.overlay_759286.id(),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: || &SCREEN_FLOW.scene_list, // g_sceneList
            table: || DRAW_LISTS.redefine_keys_player1_scene.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.draw_list, // g_drawList
            table: || DRAW_LISTS.draw_758de2.id(),
            run: false,
        },
        List {
            pointer: || &SCREEN_FLOW.overlay_list, // g_overlayList
            table: || DRAW_LISTS.overlay_759286.id(),
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
        Step::ArgText(|| &crate::menu::texts_store::MENU_TEXTS.redefine_keys_player1_picture),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
    ],
    after_first_frame: &[Step::DoFlow(after_first_frame)],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[],
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
    screen_entry("menu::redefine_keys_player1_loop", &SCREEN)
}
