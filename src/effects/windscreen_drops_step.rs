//! `effects::windscreen_drops_step` (0x4BCAD0): the steps since the windscreen drops were last looked at
//! (`dword_814714` less `dword_84460C`, low byte) into `dword_81622E`, `dword_84460C` the
//! clock `dword_814714`; `effects::expire_windscreen_drops`; then `dword_84460C` moved on by those
//! steps (low byte) and back into `dword_814714`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BAC87` (0x4BAF7D). The call is `expire_windscreen_drops`' Rust version.

use crate::effects::expire_windscreen_drops::expire_windscreen_drops;
use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::Refused;

/// The Rust version (see the header).
// Converted: `rc2k_core::effects::steps::steps`
pub fn windscreen_drops_step(masks: &Masks, drops: &WindscreenDrops) -> Result<(), Refused> {
    {
        let clock: u32 = drops.state.clock.bits();
        let steps = rc2k_core::effects::steps::steps(drops.picks.index.get(), clock);
        drops.picks.index.set(clock);
        drops.hex.steps.set(steps);
        expire_windscreen_drops(masks, drops)?;
        let latest = drops.picks.index.get().wrapping_add(drops.hex.steps.get());
        drops.picks.index.set(latest & 0xFF);
        drops.state.clock.set_bits(latest & 0xFF);
    }
    Ok(())
}
