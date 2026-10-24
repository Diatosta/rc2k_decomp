//! `menu::rank_time_trial_best` (0x48CC51): the tenth of the chosen stage's best times
//! moved up to its place: from the tenth to the second, each entry whose +0x14 (the time)
//! is below (unsigned) the one before swaps with it, field by field: +0xC, +0x14, +4, +8,
//! +0x1C, +0x10, +0x18, +0x40, +0x50, +0x54, +0x58, the dwords +0x5C .. +0x98, and the 31
//! name bytes at +0x20 (through `g_keptPlayerName`, which keeps the last swap's upper
//! name). The pass does not stop at the first entry in order.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `menu::enter_time_trial_best`. Refuses (before anything) what `event_flow::stage_times`
//! refuses.

use crate::errors::Refused;
use crate::menu::event_flow::*;

// g_keptPlayerName
/// The chosen stage's ten best times, for the core; refused as `event_flow::stage_times`.
pub fn stage_list<'a>() -> Result<rc2k_core::menu::standings::Entries<'a>, Refused> {
    let times = stage_times()?;
    Ok(unsafe { crate::menu::standings::table(times, TIME_ENTRY, 10) })
}

pub fn rank_time_trial_best() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::bests::rank_time_trial`.
    let mut list = stage_list()?;
    rc2k_core::menu::bests::rank_time_trial(&mut list, crate::menu::standings::kept_name());
    Ok(())
}
