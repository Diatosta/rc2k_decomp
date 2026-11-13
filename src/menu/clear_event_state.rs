//! `menu::clear_event_state` (0x486C2C): an event's state cleared at its start.
//!
//! `event.stages_done`, `results.round` and the 28 dwords `results.time_bonus` .. the fourth player's total set to 0 (the
//! results screens and the savegame read them; their meaning is not read). Callers: the event loops
//! (`menu::single_rally` and four more).
use crate::cells::Store;
use crate::menu::owner::Menus;

pub fn clear_event_state(menus: &Menus) -> Result<(), crate::errors::BadInput> {
    menus.screens.event.stages_done.store(0, &[0; 4])?;
    menus.play.results.round.store(0, &[0; 4])?;
    let results = &menus.play.results;
    for run in [
        &results.time_bonus,
        &results.positions,
        &results.points,
        &results.extra,
        &results.sums,
        &results.before,
        &results.totals,
    ] {
        for cell in run {
            cell.set(0);
        }
    }
    Ok(())
}
