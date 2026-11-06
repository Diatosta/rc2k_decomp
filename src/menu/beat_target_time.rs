//! `menu::beat_target_time` (0x4A3C23): `dword_755918` 1 when the current player's
//! (`dword_75598C`) time (+0x88) is at most the current event's target (its +0x57C +
//! 0x14; signed), else 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A3BE0`. Refuses a record past the table, no events table, or a current player past
//! 3 or missing, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::standings::*;

/// The Rust version (see the header).
pub fn beat_target_time() -> Result<(), Refused> {
    let event = current_event()?;
    let n: u32 = crate::menu::owner::menus()
        .screens
        .event
        .current_player
        .get();
    if n >= 4 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    // Converted: `rc2k_core::menu::bests::makes_time`.
    let target = with_event(event, |ten| ten[9].head.key)?;
    let beaten = rc2k_core::menu::bests::makes_time(player(n).get().time, target);
    crate::menu::owner::menus()
        .screens
        .event
        .time_qualifies
        .set(u32::from(beaten));
    Ok(())
}
