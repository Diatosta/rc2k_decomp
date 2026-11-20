//! One of the lobby's buttons, drawn right to left with `net::draw_button`; dim while this
//! player is ready (except item 3) and for item 0 after a race. x moves left past the button.

use crate::errors::Refused;
use crate::net::draw_button::draw_button;
use crate::net::owner::Net;
use rc2k_core::net::screens;

/// The button `string` for item `item` at (`x`, `y`); returns the next
/// button's x.
#[allow(
    clippy::too_many_arguments,
    reason = "the button's string, item and place, and the owners"
)]
pub(super) fn lobby_button(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    net: &Net,
    string: u32,
    item: u32,
    x: u32,
    y: u32,
    gfx: &'static crate::gfx::owner::Gfx,
    menus: &crate::menu::owner::Menus,
) -> Result<u32, Refused> {
    // The logic is in `rc2k_core::net::screens` (`lobby_button_flags`, `next_button_x`).
    let mut flags =
        screens::lobby_button_flags(net.vars.own.state(), item, net.lobby().after_race.get());
    // The computer drivers' switch is the host's, and not for changing while ready.
    if (item == screens::BOTS_ITEM || item == screens::START_ITEM)
        && (!net.vars.own.state().is_host() || net.vars.own.state().is_ready())
    {
        flags |= screens::DIM;
    }
    draw_button(
        so,
        net,
        &net.lobby().edit,
        string,
        flags,
        x,
        y,
        item,
        gfx,
        menus,
    )?;
    Ok(screens::next_button_x(x, net.button.get().width.get()))
}
