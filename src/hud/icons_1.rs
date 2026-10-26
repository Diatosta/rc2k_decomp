//! `hud::icons_1` (0x4B5839): layout 1's icons: the scale `dword_75C0C0`; the first entry shifted -0xC8, at 0xC8; the texture on one (`dword_75C034`); the second shifted 0xC8, at 0xC8.
//!
//! In:  ebp the HUD's record
//! Out: as the last `draw_icon` leaves them

use crate::errors::Refused;
use crate::hud::draw_icon::draw_icon;
use crate::hud::store::HUD;

/// The Rust version (see the header).
pub fn icons_1(record: u32) -> Result<(), Refused> {
    HUD.icon_scale.set_bits(HUD.icon_scales[2].bits());
    HUD.icon_shift.set(0xFFFF_FF38);
    { draw_icon(record, 0xC8, 0) }?;
    HUD.texture.set(HUD.texture.get().wrapping_add(1));
    HUD.icon_shift.set(0xC8);
    draw_icon(record, 0xC8, 0x10)
}
