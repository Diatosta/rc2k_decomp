//! `hud::icons_2` (0x4B57FE): layout 2's icons: no shift, the scale `dword_75C0BC`, the first entry at 0x3C, the texture on one (`dword_75C034`), the second at 0x177.
//!
//! In:  ebp the HUD's record
//! Out: as the last `draw_icon` leaves them

use crate::errors::Refused;
use crate::hud::draw_icon::draw_icon;
use crate::hud::owner::Hud;
use crate::pointers::GamePtr;
use crate::race::owner_frame::Frame;

/// The Rust version (see the header).
pub fn icons_2(frame: &Frame, hud: &Hud, record: GamePtr) -> Result<(), Refused> {
    hud.block.icon_shift.set(0);
    hud.block
        .icon_scale
        .set_bits(hud.block.icon_scales[1].bits());
    { draw_icon(frame, hud, record, 0x3C, 0) }?;
    hud.block
        .texture
        .set(hud.block.texture.get().wrapping_add(1));
    draw_icon(frame, hud, record, 0x177, 0x10)
}
