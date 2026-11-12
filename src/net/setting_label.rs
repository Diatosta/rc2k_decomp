//! `net::setting_label` (0x416AF8): a setting's label as a button (`net::draw_button`, flags
//! 0, x 0x190, at y), and y 0x20 lower. (The code after its `ret`, 0x416B0D .. 0x416B52,
//! is reached by nothing.)
//!
//! Takes the string, the item, y. Returns 0, 0x190, y, + 0x20.
//! Caller: `net::settings_frame` (four times).

use crate::errors::Refused;
use crate::install::owner::Install;
use crate::net::draw_button::draw_button;
use crate::net::owner::Net;

/// The label `string` for item `item` at `y`; returns the next y (y + 0x20).
pub fn setting_label(
    net: &Net,
    install: &Install,
    string: u32,
    item: u32,
    y: u32,
    gfx: &crate::gfx::owner::Gfx,
    menus: &crate::menu::owner::Menus,
) -> Result<u32, Refused> {
    draw_button(net, install, string, 0, 0x190, y, item, gfx, menus)?;
    Ok(y.wrapping_add(0x20))
}
