//! `screens::draw_background_band` (0x4698C4): draws a band of the background again.
//!
//! In:  ebx (the sprite's palette)
//! Out: every register preserved
//! Only while `dword_7554E4` is 1: sprite 0xA0 of the sprite table (the background
//! picture) at (0, 0) with the clip rectangle's top and bottom at 0xCB and 0x1C6 for
//! that one blit, and put back after it.
//! In 15 lists of screens (the setup and saved game lists, the service area set the
//! switch). What the band covers has not been seen: the name is a reading.
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, rva};
use crate::screens::harness::{Left, draw_entry};
use crate::text::draw_text::Canvas;

// ---- game data ----

const SWITCH_RVA: usize = rva(0x7554E4); // dword_7554E4

// ---- Rust version ----

pub unsafe fn draw_background_band(palette: u32, canvas: &mut dyn Canvas) -> Result<(), Refused> {
    // Converted: `rc2k_core::screens::controller_page::background_band`.
    use rc2k_core::screens::controller_page::{Font, Work, background_band};
    background_band(
        &mut crate::screens::game_screen::GameScreen::<Font, Work>::new(canvas),
        unsafe { game_read::<u32>(SWITCH_RVA) } == 1,
        palette,
    )
}

fn rust(r: &Regs, canvas: &mut dyn Canvas) -> Result<Left, Refused> {
    unsafe { draw_background_band(r.ebx, canvas) }?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    draw_entry("screens::draw_background_band", r, &rust)
}
