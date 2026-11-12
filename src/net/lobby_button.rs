//! `net::lobby_button` (0x41802F): one of the lobby's buttons, right to left: drawn
//! (`net::draw_button`, flags 1, at x, y), dim (flag 2) while this player is ready (state & 1)
//! for all but item 3, and for item 0 after a race (`after_race` 1); x then
//! moves left past it (its width `width` + 0x44, and 8 more). (The code after its
//! `ret`, 0x41806E .. 0x418082, is reached by nothing.)
//!
//! Takes the string, the item, x and y.
//! Caller: `net::lobby_frame` (four times).

use crate::errors::Refused;
use crate::install::owner::Install;
use crate::net::draw_button::draw_button;
use crate::net::owner::Net;
use rc2k_core::net::screens;

/// The button `string` for item `item` at (`x`, `y`); returns the next
/// button's x.
#[allow(
    clippy::too_many_arguments,
    reason = "the button's string, item and place, and the owners"
)]
pub fn lobby_button(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    net: &Net,
    install: &Install,
    string: u32,
    item: u32,
    x: u32,
    y: u32,
    gfx: &crate::gfx::owner::Gfx,
    menus: &crate::menu::owner::Menus,
) -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::screens` (`lobby_button_flags`, `next_button_x`).
    let flags = screens::lobby_button_flags(net.vars.own.state(), item, net.vars.after_race.get());
    draw_button(so, net, install, string, flags, x, y, item, gfx, menus)?;
    Ok(screens::next_button_x(x, net.button.get().width.get()))
}
