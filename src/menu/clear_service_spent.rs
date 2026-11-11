//! `menu::clear_service_spent` (0x4A969C): the four players' +0x288 0.
//!
//! Refuses a missing player, before anything.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;

pub fn clear_service_spent(menus: &Menus) -> Result<(), Refused> {
    for n in 0..4 {
        player(menus, n).update(|q| q.service_spent = 0);
    }
    Ok(())
}
