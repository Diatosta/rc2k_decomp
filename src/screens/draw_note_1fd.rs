//! `screens::draw_note_1fd` (0x4881F8): string 0x1FD drawn at (0x19A, 0x118) in the font at
//! 0x51E8A4 (`text::draw_string_by_id`).
//!
//! In:  nothing
//! Out: as `draw_string_by_id` leaves them
//!
//! In the draw list 0x758C52 of `menu::unused_cspecs_loop` (0x4AA1A3), which nothing reaches
//! (no code or data refers to it, 2026-10-20). Hook removed 2026-10-22 (typed tables): the
//! menus and draw lists run its `entry` (`menu::table_calls`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

/// The Rust version: the outcome's registers.
pub unsafe fn draw_note_1fd(canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    // Converted: `rc2k_core::screens::notes::NOTE_1FD`.
    crate::screens::game_screen::draw_note(canvas, &rc2k_core::screens::notes::NOTE_1FD)
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_note_1fd(canvas) }
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_note_1fd", r, &rust)
}
