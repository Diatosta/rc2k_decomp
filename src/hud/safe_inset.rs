//! The part of the screen the HUD keeps to where a phone's notch, rounded corners and home
//! indicator take the screen's edges: the HUD is laid out as on a screen of that part's size and
//! put where the part is, while the 3D picture behind it fills the whole frame. The whole screen
//! on any other device.

use rc2k_core::race::view::PlayerPart;

/// The usable part (x, y, width, height) of a `screen` of the given size, or of `part` of it when
/// the HUD is being drawn into one; relative to the part's corner.
pub fn get(screen: (u32, u32), part: Option<PlayerPart>) -> [u32; 4] {
    let [x, y, w, h] = crate::platform::safe_area::hud_inset(screen);
    let Some(part) = part else {
        return if w == 0 || h == 0 {
            [0, 0, screen.0, screen.1]
        } else {
            [x, y, w, h]
        };
    };
    let left = x.max(part.x);
    let top = y.max(part.y);
    let right = (x + w).min(part.x + part.width).max(left);
    let bottom = (y + h).min(part.y + part.height).max(top);
    if right == left || bottom == top {
        return [0, 0, part.width, part.height];
    }
    [left - part.x, top - part.y, right - left, bottom - top]
}
