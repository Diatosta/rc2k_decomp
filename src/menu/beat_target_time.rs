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
use crate::menu::event_store::EVENT_STATE;
use crate::menu::standings::*;

/// The Rust version (see the header).
pub fn beat_target_time() -> Result<(), Refused> {
    let event = current_event()?;
    let n: u32 = { EVENT_STATE.current_player.get() };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    // Converted: `rc2k_core::menu::bests::makes_time`.
    let beaten =
        rc2k_core::menu::bests::makes_time(get(player(n) + 0x88)?, get(event + 0x57C + 0x14)?);
    EVENT_STATE.time_qualifies.set(u32::from(beaten));
    Ok(())
}
