//! `menu::redefine_keys_player4_loop` (0x479F52): the keyboard assignment screen "REDEFINIR
//! TECLADO" of the fourth car.
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
use crate::menu::screen_items_store::slot;
use crate::menu::screen_loop::{
    Callee, Entry, Flow, FrameRoutine, List, Screen, Step, quitting, screen_entry,
};

// ---- Rust version: `menu::screen_loop` with this ----

fn before(entry: &Entry) -> Result<(), Refused> {
    let menus = entry.menus;
    // The player's twelve bytes at +0x1C copied to `dword_755595` (`rep movsb` between
    // `pusha` and `popa`; eax pushed and popped around the player's address: no carried
    // value changes).
    {
        let keys = menus.play.players.players[3].get().keys;
        for (n, key) in keys.into_iter().enumerate() {
            menus.keys.tables.redefined_keys.set(n, key);
        }
        menus.screens.event.current_player.set(0x3u32);
    }
    Ok(())
}

fn after_first_frame(_: &Entry) -> Result<Flow, Refused> {
    if quitting() as u32 == 0x1 {
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

fn each_frame(entry: &Entry) -> Result<Flow, Refused> {
    let screens = &entry.menus.screens;
    if screens.option_work.redefining_key.get() == 0x1
        && screens.option_work.redefining_key.get() != 0x0
    {
        return Ok(Flow::NextFrame);
    }
    Ok(Flow::On)
}

pub const SCREEN: Screen = Screen {
    menu: |m| m.tables.menus.redefine_keys_player4.id(),
    saved_item: |m| &m.screens.items.current[slot::REDEFINE_KEYS_PLAYER4],
    lists: &[
        List {
            pointer: |m| &m.screens.flow.unread_list, // g_unreadList
            table: |m| m.tables.draw_lists.unread_758b72.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.draw_758e06.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.overlay_759292.id(),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: |m| &m.screens.flow.scene_list, // g_sceneList
            table: |m| m.tables.draw_lists.redefine_keys_player4_scene.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.draw_list, // g_drawList
            table: |m| m.tables.draw_lists.draw_758e06.id(),
            run: false,
        },
        List {
            pointer: |m| &m.screens.flow.overlay_list, // g_overlayList
            table: |m| m.tables.draw_lists.overlay_759292.id(),
            run: false,
        },
    ]),
    frame_callback: FrameRoutine::Menu,
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[],
    background: &[Step::Call(Callee::LoadBackground(|m| {
        &m.tables.texts.redefine_keys_player4_picture
    }))],
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
    screen_entry("menu::redefine_keys_player4_loop", &SCREEN)
}
