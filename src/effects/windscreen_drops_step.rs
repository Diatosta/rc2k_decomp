//! `effects::windscreen_drops_step` (0x4BCAD0): the steps since the windscreen drops were last
//! looked at (the last-seen clock less the picks' clock, low byte) kept, the picks' clock the
//! last-seen clock; `effects::expire_windscreen_drops`; then the picks' clock moved on by those
//! steps (low byte) and back into the last-seen clock.
//!
//! Caller: `effects::windscreen_drops_top`. The call is `expire_windscreen_drops`.
//! version.

use crate::effects::expire_windscreen_drops::expire_windscreen_drops;
use crate::effects::mask_store::Masks;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::errors::Refused;

// Converted: `rc2k_core::effects::steps::steps`
pub fn windscreen_drops_step(masks: &Masks, drops: &WindscreenDrops) -> Result<(), Refused> {
    {
        let clock: u32 = drops.state.clock.get();
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
