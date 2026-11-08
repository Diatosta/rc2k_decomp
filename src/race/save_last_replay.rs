//! `race::save_last_replay` (0x41ECFF): the last race's replay saved as replay eax: opened
//! (`race::open_viewed_replay`, 222), saved (`race::save_viewed_replay`, ebx the slot) and
//! closed (`race::close_viewed_replay`).
//!
//! Takes: the slot
//!
//! Caller: `race::replay_viewer` (mode 3, the slot from `menu::save_replay_loop`; no flags
//! read).

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::viewer::LAST_RACE;

/// The Rust version. `slot` the slot to save the last race's replay into (the original's
/// eax); the carry the original leaves is read by none of its callers.
pub fn save_last_replay(race: &Race, slot: u32) -> Result<(), Refused> {
    if crate::race::open_viewed_replay::open_viewed_replay(race, LAST_RACE)? {
        return Ok(());
    }
    crate::race::save_viewed_replay::save_viewed_replay(race, slot)?;
    crate::race::close_viewed_replay::close_viewed_replay(race)
}
