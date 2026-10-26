//! `hud::icons_2` (0x4B57FE): layout 2's icons: no shift, the scale `dword_75C0BC`, the first entry at 0x3C, the texture on one (`dword_75C034`), the second at 0x177.
//!
//! In:  ebp the HUD's record
//! Out: as the last `draw_icon` leaves them

use crate::errors::Refused;
use crate::hud::draw_icon::draw_icon;
use crate::hud::store::HUD;

/// The Rust version (see the header).
pub fn icons_2(record: u32) -> Result<(), Refused> {
    HUD.icon_shift.set(0);
    HUD.icon_scale.set_bits(HUD.icon_scales[1].bits());
    { draw_icon(record, 0x3C, 0) }?;
    HUD.texture.set(HUD.texture.get().wrapping_add(1));
    draw_icon(record, 0x177, 0x10)
}
