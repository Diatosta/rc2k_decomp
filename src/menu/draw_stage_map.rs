//! `menu::draw_stage_map` (0x4683CF): the UK map with the stage, as the stage screens'
//! background.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `menu/stage_choice.rs`. 14 callers (the stage choice's settings, the stage
//! screens' backgrounds). The map that cannot be loaded raises fatal error 0x80712345.
//! Refuses a choice without its tables.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::menu::stage_choice::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { draw_stage_map(r) }?;
    Ok(())
}

// records::find_record, gfx::load_picture_with_overlay, convert_picture, picture_as_surface,
// blit_glyph.
