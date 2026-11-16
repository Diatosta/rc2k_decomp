//! `menu::inspection_faults`: the inspection's thirteen fault flags cleared, then each 1 for its
//! inspected part of the current player (`event.current_player`) at or past its limit
//! (`menu::standings::INSPECTED`, in order).
//!
//! Refuses a current player past 3 or missing, before anything.
use crate::errors::{BadInput, Refused};
use crate::memory::session::write_prefix;
use crate::menu::owner::Menus;
use crate::menu::standings::*;

pub fn inspection_faults(menus: &Menus) -> Result<(), Refused> {
    let n: u32 = menus.screens.event.current_player.get();
    if n >= 4 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    let p = player(menus, n);
    let fault = crate::menu::event_flow::Fault::default();
    let flags =
        rc2k_core::menu::inspection::faults(|k| fault.ok(past_limit(&menus.screens, p, k as u32)));
    fault.check()?;
    write_prefix(&menus.lists.inspection_faults, &flags);
    Ok(())
}
