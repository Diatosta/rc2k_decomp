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
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{Carried, List, Screen, Step, prepare_scene, screen_entry};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn prepare(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        game_write(rva(0x623FB0), 0x0u32);
        game_write(rva(0x623FB4), 0x0u32);
        game_write(rva(0x624080), 0x100u32);
        prepare_scene(c);
        game_write(rva(0x6241DC), 0x0u32);
        game_write(rva(0x6241E0), 0x320u32);
        game_write(rva(0x6241EC), game_read::<i32>(rva(0x6241DC)) as f32);
        game_write(rva(0x6241F0), game_read::<i32>(rva(0x6241E0)) as f32);
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x751DE4),
    saved_item: rva(0x74767C),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758F1A),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758BBA),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x75915E),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[],
    set_up_first: &[Step::Write(rva(0x7554E4), 0x0)],
    prepare: &[Step::Do(prepare)],
    after_lists: &[],
    background: &[],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Write(rva(0x7554E4), 0x1)],
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
    leave_saved_item: Some(rva(0x747678)),
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
