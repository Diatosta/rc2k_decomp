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
//! Callers: none in code: it is reached through a table.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::menu::screen_loop::{
    Carried, Flow, List, Screen, Step, quitting, read_at, screen_entry,
};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn before(_c: &mut Carried) -> Result<(), Refused> {
    // The player's twelve bytes at +0x1C copied to `dword_755595` (`rep movsb` between
    // `pusha` and `popa`; eax pushed and popped around the player's address: no carried
    // value changes).
    unsafe {
        let player = read_at(game_addr(rva(0x75647C)) as u32);
        std::ptr::copy(
            player.wrapping_add(0x1C) as *const u8,
            game_addr(rva(0x755595)) as *mut u8,
            0xC,
        );
        game_write(rva(0x75598C), 0x0u32);
    }
    Ok(())
}

unsafe fn after_first_frame(_c: &mut Carried) -> Result<Flow, Refused> {
    if quitting() as u32 == 0x1 {
        return Ok(Flow::Leave);
    }
    Ok(Flow::On)
}

unsafe fn each_frame(_c: &mut Carried) -> Result<Flow, Refused> {
    unsafe {
        if game_read::<u32>(rva(0x7556AC)) == 0x1 && game_read::<u32>(rva(0x7556AC)) != 0x0 {
            return Ok(Flow::NextFrame);
        }
        Ok(Flow::On)
    }
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74BB60),
    saved_item: rva(0x747748),
    lists: &[
        List {
            pointer: rva(0x7554F4), // g_unreadList
            table: rva(0x758B72),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758DE2),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759286),
            run: false,
        },
    ],
    first_frame_lists: Some(&[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x7590C2),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758DE2),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759286),
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
        Step::ArgAddr(rva(0x7322FB)),
        Step::CallOrFatal(rva(0x47EFFF), 0x80712345),
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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn run() -> Result<(), crate::errors::Refused> {
    screen_entry("menu::redefine_keys_player1_loop", &SCREEN)
}
