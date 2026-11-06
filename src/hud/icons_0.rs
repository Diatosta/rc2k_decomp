//! `hud::icons_0` (0x4B57D8): layout 0's icon: no shift, the scale `dword_75C0B8`, the first entry at 0xC8.
//!
//! In:  ebp the HUD's record
//! Out: as the last `draw_icon` leaves them

use crate::errors::Refused;
use crate::hud::draw_icon::draw_icon;
use crate::hud::owner::Hud;
use crate::pointers::GamePtr;

/// The Rust version (see the header).
pub fn icons_0(hud: &Hud, record: GamePtr) -> Result<(), Refused> {
    hud.block.icon_shift.set(0);
    hud.block
        .icon_scale
        .set_bits(hud.block.icon_scales[0].bits());
    draw_icon(hud, record, 0xC8, 0)
}
