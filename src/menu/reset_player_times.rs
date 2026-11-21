//! `menu::reset_player_times`: the four players' `time` 3600000 (an hour in
//! milliseconds: no time yet) and `entry` 6 (out); `event.event_goes_on` 0.
//!
//! Refuses a missing player, before anything.
use crate::errors::Refused;
use crate::menu::owner::{Play, Screens};
use crate::menu::standings::*;

pub fn reset_player_times(screens: &Screens, play: &Play) -> Result<(), Refused> {
    for n in 0..4 {
        player_in(play, n).update(rc2k_core::menu::player::Player::reset_times);
    }
    screens.event.write().event_goes_on = 0u32;
    Ok(())
}
