//! `hud::icons_0` (0x4B57D8): layout 0's icon: no shift, the scale `dword_75C0B8`, the first entry at 0xC8.
//!
//! In:  ebp the HUD's record
//! Out: as the last `draw_icon` leaves them

use crate::errors::Refused;
use crate::hud::draw_icon::draw_icon;
use crate::hud::store::HUD;

/// The Rust version (see the header).
pub unsafe fn icons_0(record: u32) -> Result<(), Refused> {
    HUD.icon_shift.set(0);
    HUD.icon_scale.set_bits(HUD.icon_scales[0].bits());
    unsafe { draw_icon(record, 0xC8, 0) }
}
