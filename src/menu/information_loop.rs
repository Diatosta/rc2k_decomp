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
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{
    Callee, Carried, List, Screen, Step, carousel, log_line, prepare_scene, screen_entry,
};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        // Between `pusha` and `popa`: no register changes.
        log_line(rva(0x4DEAD0));
        game_write(rva(0x623FB0), 0xFFFFFF64u32);
        game_write(rva(0x623FB4), 0x27u32);
        game_write(rva(0x624080), 0x140u32);
        prepare_scene(c);
        game_write(rva(0x6241DC), 0xFFFFFFD8u32);
        game_write(rva(0x6241E0), 0x2F8u32);
        game_write(rva(0x6241EC), game_read::<i32>(rva(0x6241DC)) as f32);
        game_write(rva(0x6241F0), game_read::<i32>(rva(0x6241E0)) as f32);
        game_write(rva(0x7554EC), 0x1u32);
        game_write(rva(0x75AA74), 0x9u32);
        carousel(c)?;
        c.arg = game_read(rva(0x7476BC));
        game_write(rva(0x755525), c.arg);
        c.arg = c.arg.wrapping_sub(1);
        game_write(rva(0x75AA7C), c.arg);
    }
    Ok(())
}

unsafe fn after(_c: &mut Carried) -> Result<(), Refused> {
    // Between `pusha` and `popa`: no register changes.
    unsafe {
        log_line(rva(0x4DEADF));
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74F90C),
    saved_item: rva(0x7476BC),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758FEE),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758C82),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591CA),
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
        Step::Write(rva(0x7558D8), 0x1),
        Step::ArgAddr(rva(0x731DB4)),
        Step::CallOrFatal(Callee::LoadBackground, 0x80712345),
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
