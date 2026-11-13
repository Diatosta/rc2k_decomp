//! A setting's label drawn as a button at x 0x190; returns the item's y moved down by 0x20.

use crate::errors::Refused;
use crate::install::owner::Install;
use crate::net::draw_button::draw_button;
use crate::net::owner::Net;

/// The label `string` for item `item` at `y`; returns the next y (y + 0x20).
pub fn setting_label(
    o: crate::owners::Owners<'static>,
    net: &Net,
    install: &Install,
    string: u32,
    item: u32,
    y: u32,
) -> Result<u32, Refused> {
    draw_button(
        crate::strings::append_string_by_id::StringOwners::of(o),
        net,
        install,
        string,
        0,
        0x190,
        y,
        item,
        o.gfx,
        o.menus,
    )?;
    Ok(y.wrapping_add(0x20))
}
