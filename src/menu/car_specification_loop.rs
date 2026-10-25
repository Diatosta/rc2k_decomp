//! `menu::car_specification_loop` (0x47A652): the screen titled "ESPECIFICAÇÕES DO CARRO".
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is not asked about; it has a step of its own once
//! a frame; one call comes after the registers are put back; the steps that are more than a
//! call or a store are the functions below, generated from the listing.
//! The name is a reading from the title string of its menu, "ESPECIFICAÇÕES DO CARRO", not yet
//! a visit.
//! Callers: none in code: it is reached through a table. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{
    Callee, Carried, List, Screen, Step, carousel, prepare_scene, screen_entry,
};
use crate::menu::state_store::MENU_STATE;

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn before(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        c.arg = game_read(rva(0x7585DC));
        game_write(rva(0x755990), c.arg);
        game_write(rva(0x755998), 0x0u32);
    }
    Ok(())
}

unsafe fn set_up_first(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        game_write(rva(0x623FB0), 0xFFFFFF60u32);
        game_write(rva(0x623FB4), 0xFFFFFF9Cu32);
        game_write(rva(0x624080), 0x100u32);
        prepare_scene(c);
        game_write(rva(0x7554EC), 0x1u32);
        MENU_STATE.carousel.menu.set(0xBu32);
        carousel(c)?;
        c.arg = game_read(rva(0x7476D4));
        game_write(rva(0x755525), c.arg);
        c.arg = c.arg.wrapping_sub(1);
        MENU_STATE.carousel.front.set(c.arg);
        game_write(rva(0x7554EC), 0x0u32);
        c.arg = game_read(rva(0x755990));
        MENU_STATE.carousel.front.set(c.arg);
    }
    Ok(())
}

unsafe fn after(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        c.arg = game_read(rva(0x755990));
        game_write(rva(0x7585DC), c.arg);
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74B628),
    saved_item: rva(0x7476D4),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x758F76),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758CDA),
            run: false,
        },
        List {
            pointer: rva(0x755500), // dword_755500
            table: rva(0x7593C6),
            run: true,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x7591FA),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[Step::Do(set_up_first)],
    prepare: &[],
    after_lists: &[],
    background: &[
        Step::Call(Callee::LoadCarBackground),
        Step::Call(Callee::PlayCarStream),
    ],
    after_first_frame: &[],
    confirm: None,
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Do(after)],
    escape_leaves: false,
    each_frame: &[Step::Call(Callee::StepCarStream)],
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
    after_restored: &[Step::Call(Callee::StopStream)],
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
    screen_entry("menu::car_specification_loop", &SCREEN)
}
