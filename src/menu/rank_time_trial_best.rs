//! `menu::rank_time_trial_best` (0x48CC51): the tenth of the chosen stage's best times
//! moved up to its place: from the tenth to the second, each entry whose +0x14 (the time)
//! is below (unsigned) the one before swaps with it, field by field: +0xC, +0x14, +4, +8,
//! +0x1C, +0x10, +0x18, +0x40, +0x50, +0x54, +0x58, the dwords +0x5C .. +0x98, and the 31
//! name bytes at +0x20 (through `g_keptPlayerName`, which keeps the last swap's upper
//! name). The pass does not stop at the first entry in order.
//!
//! Caller: `menu::enter_time_trial_best`. Refuses (before anything) what `event_flow::stage_times`
//! refuses.

use crate::errors::Refused;
use crate::menu::event_flow::*;
use crate::menu::owner::Play;

// g_keptPlayerName
pub fn rank_time_trial_best(play: &Play) -> Result<(), Refused> {
    // Converted: `rc2k_core::saves::entries::rank_time_trial`.
    Ok(with_stage_times(|list| {
        crate::menu::standings::with_kept_name(play, |kept| {
            rc2k_core::saves::entries::rank_time_trial(list, kept);
        });
    })?)
}
