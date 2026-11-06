//! `hud::icons_1` (0x4B5839): layout 1's icons: the scale `dword_75C0C0`; the first entry shifted -0xC8, at 0xC8; the texture on one (`dword_75C034`); the second shifted 0xC8, at 0xC8.
//!
//! In:  ebp the HUD's record
//! Out: as the last `draw_icon` leaves them

use crate::errors::Refused;
use crate::hud::draw_icon::draw_icon;
use crate::hud::owner::Hud;
use crate::pointers::GamePtr;

/// The Rust version (see the header).
pub fn icons_1(hud: &Hud, record: GamePtr) -> Result<(), Refused> {
    hud.block
        .icon_scale
        .set_bits(hud.block.icon_scales[2].bits());
    hud.block.icon_shift.set(0xFFFF_FF38);
    { draw_icon(hud, record, 0xC8, 0) }?;
    hud.block
        .texture
        .set(hud.block.texture.get().wrapping_add(1));
    hud.block.icon_shift.set(0xC8);
    draw_icon(hud, record, 0xC8, 0x10)
}
