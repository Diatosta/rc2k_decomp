//! `menu::inspection_faults` (0x4A1733): the inspection's thirteen fault flags
//! (`dword_759C76` ..) cleared, then each 1 for its inspected part of the current player
//! (`dword_75598C`) at or past its limit (`menu::standings::INSPECTED`, in order).
//!
//! Refuses a current player past 3 or missing,
//! before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::standings::*;
use crate::session::write_prefix;

pub fn inspection_faults(menus: &Menus) -> Result<(), Refused> {
    let n: u32 = menus.screens.event.current_player.get();
    if n >= 4 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    // Converted: `rc2k_core::menu::inspection::faults`.
    let p = player(n);
    let fault = crate::menu::event_flow::Fault::default();
    let flags =
        rc2k_core::menu::inspection::faults(|k| fault.ok(past_limit(&menus.screens, p, k as u32)));
    fault.check()?;
    write_prefix(&menus.tables.lists.inspection_faults, &flags);
    Ok(())
}
