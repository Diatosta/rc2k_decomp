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
use crate::text::draw_text::Canvas;

// ---- Rust version ----

/// Returns the carry flag of the last text drawn.
pub unsafe fn draw_empty_savegame(canvas: &mut dyn Canvas) -> Result<bool, Refused> {
    // Converted: `rc2k_core::screens::savegame_preview::empty`.
    rc2k_core::screens::savegame_preview::empty(
        &mut crate::screens::game_screen::PreviewScreen::new(canvas),
    )
}
