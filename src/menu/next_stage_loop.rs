//! `menu::next_stage_loop` (0x473B21): the stage details screen before a stage of a
//! championship, by its title "ETAPA SEGUINTE" and its draw list, which is that of
//! `screens::draw_stage_info` (a reading)
//!
//! In:  nothing
//! Out: every register preserved
//! The loop itself is `menu::screen_loop`; this file says what is this screen's own (`SCREEN`),
//! as extracted from the disassembly: leaving is asked about; it has a step of its own once a
//! frame; the steps that are more than a call or a store are the functions below, generated
//! from the listing.
//! The name is a reading from the title string of its menu, "ETAPA SEGUINTE", not yet a visit.
//! Callers: `470393`, `sub_4A0E60`, `sub_4AA4D7`. The menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::menu::screen_loop::{Callee, Carried, List, Screen, Step, screen_entry};

// ---- Rust version: `menu::screen_loop` with this ----

unsafe fn before(c: &mut Carried) -> Result<(), Refused> {
    unsafe {
        c.measure_x = game_read(rva(0x747904));
        game_write(rva(0x747768), c.measure_x);
    }
    Ok(())
}

pub const SCREEN: Screen = Screen {
    menu: rva(0x74DA3C),
    saved_item: rva(0x747768),
    lists: &[
        List {
            pointer: rva(0x755504), // g_sceneList
            table: rva(0x759092),
            run: false,
        },
        List {
            pointer: rva(0x7554F8), // g_drawList
            table: rva(0x758D7E),
            run: false,
        },
        List {
            pointer: rva(0x7554FC), // g_overlayList
            table: rva(0x759256),
            run: false,
        },
    ],
    first_frame_lists: None,
    frame_callback: rva(0x474D4B),
    later_frame_callback: None,
    before: &[Step::Do(before)],
    set_up_first: &[],
    prepare: &[],
    after_lists: &[Step::Write(rva(0x755700), 0x3)],
    background: &[Step::Call(Callee::DrawStageMap)],
    after_first_frame: &[Step::Call(Callee::PlayStageSpeech)],
    confirm: Some(&[]),
    ask_also_if: None,
    confirmed: &[],
    unasked_skips_save: false,
    after: &[Step::Call(Callee::StopStream)],
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
    screen_entry("menu::next_stage_loop", &SCREEN)
}
