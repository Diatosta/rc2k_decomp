//! `hud::icons_2` (0x4B57FE): layout 2's icons: no shift, the scale `dword_75C0BC`, the first entry at 0x3C, the texture on one (`dword_75C034`), the second at 0x177.
//!
//! In:  ebp the HUD's record
//! Out: as the last `draw_icon` leaves them
//!
//! The icons through `part` (1), in the game `draw_icon`'s Rust version. Shadow mode runs
//! the original alone (it draws). Self-test: a copy whose calls go to a stand-in, against
//! the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::hud::batch::TEXTURE_RVA;
use crate::hud::draw_icon::{SCALE_RVA, SHIFT_RVA, draw_icon};

/// The Rust version (see the header).
pub unsafe fn icons_2(record: u32) -> Result<(), Refused> {
    unsafe {
        game_write(SHIFT_RVA, 0u32);
        game_write(SCALE_RVA, game_read::<u32>(rva(0x75C0BC)));
        draw_icon(record, 0x3C, 0)?;
        game_write(TEXTURE_RVA, game_read::<u32>(TEXTURE_RVA).wrapping_add(1));
        draw_icon(record, 0x177, 0x10)
    }
}
