//! `effects::pick_windscreen_drop` (0x4BC6F1): one step of the marks' random (`g_marksSeed`, times
//! 0x9D plus 0x47) split into a place (bits 0 .. 10, 11 .. 20) and a kind (bits 22 .. 24)
//! for `effects::spawn_windscreen_drop`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `effects::windscreen_drops_spawn` (twice). The call is `spawn_windscreen_drop`'s Rust version.
//!
//! compared exactly as before (the earlier version stays for the Rust callers in
//! unconverted code).

use crate::effects::spawn_windscreen_drop::spawn_windscreen_drop;
use crate::effects::work_store::MARKS_WORK;
use crate::errors::Refused;

/// The Rust version (see the header).
pub fn pick_windscreen_drop() -> Result<(), Refused> {
    {
        let s = MARKS_WORK.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
        MARKS_WORK.seed.set(s);
        spawn_windscreen_drop(
            s & 0x7FF,
            s.rotate_right(11) & 0x3FF,
            s.rotate_right(22) & 7,
        )?;
    }
    Ok(())
}
