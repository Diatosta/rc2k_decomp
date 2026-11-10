//! `menu::inspection_verdict` (0x4A1952): the current player's (`dword_75598C`) inspection
//! verdict. The penalties (`dword_75AC48` ..) of its inspected parts at or past their
//! limits whose flag in `dword_759B6E` (index + 2) is 0 added into `dword_7559EC`; 100
//! less that into `dword_75AC80`; the player's +0xD0 1 when that is at least 100 (no
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
    // Converted: `rc2k_core::menu::inspection::verdict`.
    let p = player(n);

    let fault = crate::menu::event_flow::Fault::default();
    let v = rc2k_core::menu::inspection::verdict(
        |k| fault.ok(past_limit(&menus.screens, p, k as u32)),
        |k| menus.tables.lists.inspection_flags_a[k + 2].get() != 0,
        |k| menus.screens.state.driver_stats.penalties[k].get(),
    );
    fault.check()?;
    menus.screens.draw_work.figure.set(v.penalties);
    menus.screens.state.driver_stats.score.set(v.score);
    p.update(|q| q.entry = v.kind);

    Ok(())
}
