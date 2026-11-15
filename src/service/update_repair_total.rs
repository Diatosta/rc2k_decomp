//! Adds up the times of the chosen items into the repairs' total and, if the time budget minus
//! that total is at least 900, clears the second flag table (`clear_repair_marks`).
//!
//! Refuses what `state::chosen_repairs_time` refuses.

use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::service::owner::service;
use crate::service::state::{flags, marks, store_marks, time_of};

pub fn update_repair_total(menus: &Menus) -> Result<(), Refused> {
    let screens = &menus.screens;
    let mut m = marks();
    let budget = screens.event.service.minutes.get() as i32;
    let total = rc2k_core::service::repairs::update_total(&flags(), &mut m, budget, time_of)
        .map_err(|e| crate::errors::BadInput(e.0))?;
    store_marks(&m);
    service().repair_time.set(total as u32);
    Ok(())
}
