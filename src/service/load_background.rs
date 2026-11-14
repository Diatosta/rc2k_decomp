//! The service area's picture for the current player's car (`files\g\graphics\screens\servNN.png`,
//! 60 of them; car 0x21 takes car 0x17's picture), loaded and converted. A failed conversion
//! is fatal error 0x80712435.
//!
//! Called by the service area's screens. Refuses (before anything) a current player past the
//! four or missing, and a car past the table.

use crate::errors::{BadInput, Refused};
use crate::menu::lists_store::lists;
use crate::menu::owner::Menus;

/// `gfx::convert_picture` refuses instead of failing, so fatal error 0x80712435 never arises.
pub fn load_background_in(
    menus: &'static Menus,
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    let Some(player) = crate::menu::player::player(menus, menus.screens.event.current_player.get())
    else {
        return Err(BadInput("no current player (or one past the four)").into());
    };
    let car =
        rc2k_core::service::car::background(player.get().car_number).map_err(BadInput::from)?;
    let path = crate::menu::texts_store::text_at(lists().service_pictures[car as usize].get())?;
    crate::gfx::load_picture::load_picture_with(&path, gfx, system)?;
    crate::gfx::convert_picture::convert_picture_now_in(gfx, system)?;
    Ok(())
}
