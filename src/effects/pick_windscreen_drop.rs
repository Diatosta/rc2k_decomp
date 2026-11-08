//! `effects::pick_windscreen_drop` (0x4BC6F1): one step of the marks' random (`g_marksSeed`, times
//! 0x9D plus 0x47) split into a place (bits 0 .. 10, 11 .. 20) and a kind (bits 22 .. 24)
//! for `effects::spawn_windscreen_drop`.
//!
//! Caller: `effects::windscreen_drops_spawn` (twice). The call is `spawn_windscreen_drop`'s Rust version.
//!
//! compared exactly as before (the earlier version stays for the Rust callers in
//! unconverted code).

use crate::effects::spawn_windscreen_drop::spawn_windscreen_drop;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::Refused;

/// The Rust version (see the header).
pub fn pick_windscreen_drop(
    drops: &WindscreenDrops,
    effects: &crate::effects::owner::Effects,
) -> Result<(), Refused> {
    let setup = &effects.setup;
    {
        let s = setup.work.seed.get().wrapping_mul(0x9D).wrapping_add(0x47);
        setup.work.seed.set(s);
        spawn_windscreen_drop(
            drops,
            s & 0x7FF,
            s.rotate_right(11) & 0x3FF,
            s.rotate_right(22) & 7,
        )?;
    }
    Ok(())
}
