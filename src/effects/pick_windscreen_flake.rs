//! `effects::pick_windscreen_flake` (0x4C0623): one step of the windscreen flakes' own random (`dword_8BC256`,
//! times 0xC5 plus 0x5D) split into a place (bits 0 .. 10, 11 .. 20) and a kind (bits 21 ..
//! 23) for `effects::spawn_windscreen_flake`.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `effects::windscreen_flakes_spawn` (twice). The call is `spawn_windscreen_flake`'s Rust version.
//!
//! compared exactly as before (the earlier version stays for the Rust callers in
//! unconverted code).

use crate::effects::spawn_windscreen_flake::spawn_windscreen_flake;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;
use crate::race::owner_shared::Shared;

/// The Rust version (see the header).
pub fn pick_windscreen_flake(shared: &Shared, fl: &WindscreenFlakes) -> Result<(), Refused> {
    {
        let s = fl
            .state
            .pick_random
            .get()
            .wrapping_mul(0xC5)
            .wrapping_add(0x5D);
        fl.state.pick_random.set(s);
        spawn_windscreen_flake(shared, fl, s & 0x7FF, (s >> 11) & 0x3FF, (s >> 21) & 7)?;
    }
    Ok(())
}
