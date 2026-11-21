//! `sound::drop_cue`: one cue taken out of a sound object's queue.
//!
//! Nothing for a place at or past the count; else the count goes down by one and the cues after the
//! place move up. Refuses (before anything) a count over 16 (the move would run past the object).

use crate::errors::{BadInput, Refused};
use rc2k_core::sound::cars::{CueQueue, drop_cue as drop_cue_in};

/// The logic is `rc2k_core::sound::cars::drop_cue`. The cue at `place` dropped from the queue
/// `q`.
pub fn drop_cue(q: &mut CueQueue, place: u32) -> Result<(), Refused> {
    let mut cues = *q;
    drop_cue_in(&mut cues, place).map_err(|e| BadInput(e.0))?;
    *q = cues;
    Ok(())
}
