//! `menu::clear_service_spent`: the four players' +0x288 set to 0.
//!
//! Refuses a missing player, before anything.

use crate::errors::Refused;
use crate::menu::owner::Play;
use crate::menu::standings::*;

pub fn clear_service_spent(play: &Play) -> Result<(), Refused> {
    for n in 0..4 {
        player_in(play, n).update(|q| q.service_spent = 0);
    }
    Ok(())
}
