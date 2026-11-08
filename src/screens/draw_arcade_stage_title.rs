//! `screens::draw_arcade_stage_title` (0x488F2F): the arcade's after-stage screen's title:
//! string 0x1FC at (0x19A, 0x118) with the font at 0x51E8A4. In that screen's draw list
//! (0x758E66). IDA had it as code outside any function (made one 2026-10-19).
//!
//!
//! The menus and draw lists run its `entry` (`menu::table_calls`).

use crate::errors::Refused;
use crate::text::draw_text::Canvas;

// ---- Rust version ----

pub fn draw_arcade_stage_title(canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::notes::ARCADE_STAGE_TITLE`; the registers are kept (`pusha`).
    crate::screens::game_screen::draw_note(canvas, &rc2k_core::screens::notes::ARCADE_STAGE_TITLE)?;
    Ok(())
}
