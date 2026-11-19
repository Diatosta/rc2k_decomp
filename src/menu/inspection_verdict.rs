//! `menu::inspection_verdict`: the current player's (`event.current_player`) inspection
//! verdict. The penalties (the penalties ..) of its inspected parts at or past their
//! limits whose flag in the service box flags (index + 2) is 0 added up; 100
//! less that into the inspection score; the player's +0xD0 1 when that is at least 100 (no
//! penalty), else 2.
//!
//! Refuses a current player past 3 or missing, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::owner::Menus;
use crate::menu::standings::*;

pub fn inspection_verdict(menus: &Menus) -> Result<(), Refused> {
    let n: u32 = menus.screens.event.current_player.get();
    if n >= 4 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    let p = player(menus, n);

    let fault = crate::errors::Fault::default();
    let v = rc2k_core::menu::inspection::verdict(
        |k| fault.ok(past_limit(&menus.screens, p, k as u32)),
        |k| menus.service.repair_flags[k + 2].get() != 0,
        |k| menus.screens.state.driver_stats.penalties[k].get(),
    );
    fault.check()?;
    menus.screens.state.driver_stats.score.set(v.score);
    p.update(|q| q.entry = v.kind);

    Ok(())
}
