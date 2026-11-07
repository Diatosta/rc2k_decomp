//! `service::load_background` (0x470F55): the service area's picture for the current
//! player's car: the player (`g_players`[`dword_75598C`]), its car (+8; car 0x21 takes car
//! 0x17's picture), the picture `off_75A52E`[car] (`files\g\graphics\screens\servNN.png`, 60
//! of them) loaded (`gfx::load_picture`) and converted (`gfx::convert_picture`); a failed
//! conversion (carry) is fatal error 0x80712435.
//!
//!
//! Callers: the service area's screens (0x47296A, 0x472B53, 0x4A1C0F). Its calls: the Rust
//! versions. Refuses (before anything) a current player past the four or missing, and a car past
//! the table.

use crate::errors::{BadInput, Refused};
use crate::menu::owner::menus;

/// The Rust version (see the header). The conversion's carry (fatal error 0x80712435) is
/// never set by `gfx::convert_picture`'s Rust version, which refuses instead.
pub fn load_background() -> Result<(), Refused> {
    let menus = menus();
    let Some(player) = crate::menu::player::player(menus.screens.event.current_player.get()) else {
        return Err(BadInput("no current player (or one past the four)").into());
    };
    // Converted: `rc2k_core::service::car::background`.
    let car =
        rc2k_core::service::car::background(player.get().car_number).map_err(BadInput::from)?;
    let path =
        crate::menu::texts_store::text_at(menus.tables.lists.service_pictures[car as usize].get())?;
    crate::gfx::load_picture::load_picture(&path)?;
    crate::gfx::convert_picture::convert_picture_now()?;
    Ok(())
}
