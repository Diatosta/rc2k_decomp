//! `hud::icons_0` (0x4B57D8): layout 0's icon: no shift, the scale `dword_75C0B8`, the first entry at 0xC8.
//!
//! In:  ebp the HUD's record
//! Out: as the last `draw_icon` leaves them

use crate::errors::Refused;
use crate::hud::draw_icon::{SCALE_RVA, SHIFT_RVA, draw_icon};
use crate::hud::store::Slot;
use crate::hud::store::{game_read, game_write};

/// The Rust version (see the header).
pub unsafe fn icons_0(record: u32) -> Result<(), Refused> {
    unsafe {
        game_write(SHIFT_RVA, 0u32);
        game_write(SCALE_RVA, game_read::<u32>(Slot::at(0x75C0B8)));
        draw_icon(record, 0xC8, 0)
    }
}
