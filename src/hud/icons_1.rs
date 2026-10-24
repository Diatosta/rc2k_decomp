//! `hud::icons_1` (0x4B5839): layout 1's icons: the scale `dword_75C0C0`; the first entry shifted -0xC8, at 0xC8; the texture on one (`dword_75C034`); the second shifted 0xC8, at 0xC8.
//!
//! In:  ebp the HUD's record
//! Out: as the last `draw_icon` leaves them

use crate::errors::Refused;
use crate::hud::batch::TEXTURE_RVA;
use crate::hud::draw_icon::{SCALE_RVA, SHIFT_RVA, draw_icon};
use crate::hud::store::Slot;
use crate::hud::store::{game_read, game_write};

/// The Rust version (see the header).
pub unsafe fn icons_1(record: u32) -> Result<(), Refused> {
    unsafe {
        game_write(SCALE_RVA, game_read::<u32>(Slot::at(0x75C0C0)));
        game_write(SHIFT_RVA, 0xFFFF_FF38u32);
        draw_icon(record, 0xC8, 0)?;
        game_write(TEXTURE_RVA, game_read::<u32>(TEXTURE_RVA).wrapping_add(1));
        game_write(SHIFT_RVA, 0xC8u32);
        draw_icon(record, 0xC8, 0x10)
    }
}
