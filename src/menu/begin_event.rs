//! `menu::begin_event`: the drivers and the four players made ready for a new event.
//!
//! `champ_view.services` and `event.stages_done` 0; every driver (`standings::drivers`'s walk) gets
//! +0x10, +0xC, +0x20 = 0 and +0x24 = 1 (`Driver::begin`); `event.event_goes_on` 1,
//! `event.event_out` 0; then for each player n (the current player `event.current_player` = n, left
//! at 3): `total` and `unread_84` 0, `entry` 1 (`Player::begin`), `service::repair_car`, and unless
//! the event mode is 3 the setup's defaults: gear ratio 2, brake balance 0x32, tyre 3, ride height
//! 0x46, suspension 0x1E, steering sensitivity = the controller's (+0x280). Refuses drivers without
//! their end entry, a missing player.

use crate::errors::Refused;
use crate::menu::owner::{Cars, Play, Screens};
use crate::menu::player::PLAYERS;
use rc2k_core::menu::event::KEEPS_SETUP;
use rc2k_core::menu::player::Player;

pub fn begin_event(cars: &Cars, play: &Play, screens: &Screens) -> Result<(), Refused> {
    let drivers = crate::menu::standings::drivers_of(cars)?;
    play.champ_view.services.set(0u32);
    screens.event.stages_done.set(0u32);
    drivers
        .iter()
        .for_each(crate::menu::drivers_store::Driver::begin);
    screens.event.event_goes_on.set(1u32);
    screens.event.event_out.set(0u32);
    for n in 0..PLAYERS {
        screens.event.current_player.set(n);
        let p = crate::menu::standings::player_in(play, n);
        p.update(Player::begin);
        crate::service::repair_parts::repair_car_of(play, screens)?;
        if play.champ_view.event_mode.get() != KEEPS_SETUP {
            p.update(Player::default_setup);
        }
    }
    Ok(())
}
