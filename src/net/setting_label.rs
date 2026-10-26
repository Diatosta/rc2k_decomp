//! `net::setting_label` (0x416AF8): a setting's label as a button (`net::draw_button`, flags
//! 0, x 0x190, y ebp), and ebp 0x20 lower. (The code after its `ret`, 0x416B0D .. 0x416B52,
//! is reached by nothing.)
//!
//! In:  the string, the item, y
//! Out: ebx = 0, ecx = 0x190, edx = the y, ebp + 0x20; the rest as the call leaves it (it
//!      keeps every register); no `pusha`
//! Caller: `net::settings_frame` (four times).

use crate::errors::Refused;
use crate::net::draw_button::draw_button;

/// The Rust version: the label `string` for item `item` at `y`; returns the next y (the
/// original's ebp + 0x20; its other registers are kept).
pub fn setting_label(string: u32, item: u32, y: u32) -> Result<u32, Refused> {
    draw_button(string, 0, 0x190, y, item)?;
    Ok(y.wrapping_add(0x20))
}
