//! `menu::clear_result_slots`: the four players' fourteen result slots
//! (`stage_points`, `overall_points`) and every driver's (`results`) emptied: -1, but 0 in the seventh
//! and the last (two groups of six sections' points and their total).
//!
//! Refuses a missing player or a drivers' table without its end entry, before anything.
use crate::errors::Refused;
use crate::menu::drivers_store::Driver;
use crate::menu::owner::Menus;
use crate::menu::standings::*;

pub fn clear_result_slots(menus: &Menus) -> Result<(), Refused> {
    let count = walked(&menus.cars)?;
    for n in 0..4 {
        player(menus, n).update(rc2k_core::menu::player::Player::clear_results);
    }
    menus.cars.drivers.each(count, Driver::clear_results);
    Ok(())
}
