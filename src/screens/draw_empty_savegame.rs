//! `screens::draw_empty_savegame` (0x46E2DE): the load-game screen's preview for a slot with
//! no saved game: the headings and row numbers, "no entry" (string 0x578) as the title and as
//! every row's name, and "nothing" (string 0x57A) in the six other places of every row.
//!
//! In:  nothing
//! Out: every register preserved; carry as the last `draw_string_by_id` leaves it
//! Called by `draw_savegame_preview`. String 0x578 is "n/d" and string 0x57A is "-" (seen on
//! the rows of `draw_savegame_details` in the owner's screenshot of 2026-10-10); this panel
//! itself, for an empty slot, has not been seen.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::screens::font_is_loaded;
use crate::screens::harness::{Left, draw_entry, drawing_unavailable};
use crate::screens::savegame_preview::{FONTS, STRINGS};
use crate::strings::string_table::{blob, is_table_id};
use crate::strings::text_from_ids::{TEXT_LEN, text};
use crate::text::draw_text::Canvas;
use std::slice::from_raw_parts;

// ---- Rust version ----

/// Returns the carry flag of the last text drawn.
pub unsafe fn draw_empty_savegame(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    // Converted: `rc2k_core::screens::savegame_preview::empty`.
    rc2k_core::screens::savegame_preview::empty(
        &mut crate::screens::game_screen::PreviewScreen::new(canvas),
    )
}

/// The shared text buffer and the line buffer, which the drawing leaves its last texts in.
pub fn state() -> Vec<u8> {
    let mut s = unsafe { from_raw_parts(text(), TEXT_LEN) }.to_vec();
    s.extend_from_slice(unsafe { from_raw_parts(crate::screens::line_buffer(), 0x80) });
    s
}

fn rust(_: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    Ok(Left::carry(unsafe { draw_empty_savegame(canvas) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_empty_savegame", r, &rust)
}

/// Why the preview panels cannot be drawn in a test, or None.
pub fn preview_unavailable() -> Option<&'static str> {
    drawing_unavailable().or_else(|| {
        (blob().is_null()
            || !FONTS.iter().all(|&f| font_is_loaded(f))
            || !STRINGS.iter().all(|&id| is_table_id(id)))
        .then_some("the strings or the fonts are not loaded")
    })
}
