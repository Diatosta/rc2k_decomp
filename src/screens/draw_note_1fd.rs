//! `screens::draw_note_1fd` (0x4881F8): string 0x1FD drawn at (0x19A, 0x118) in the font at
//! 0x51E8A4 (`text::draw_string_by_id`).
//!
//!
//! In the draw list 0x758C52 of `menu::unused_cspecs_loop` (0x4AA1A3), which nothing reaches (no
//! code or data refers to it, 2026-10-20). The menus and draw lists run its `entry`
//! (`menu::table_calls`).

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

pub fn draw_note_1fd(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::notes::NOTE_1FD`.
    crate::screens::game_screen::draw_note(canvas, &rc2k_core::screens::notes::NOTE_1FD)?;
    Ok(())
}
