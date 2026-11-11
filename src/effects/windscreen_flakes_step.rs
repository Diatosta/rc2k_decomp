//! `effects::windscreen_flakes_step` (0x4C0654): a frame switch 0; for each step since the
//! windscreen flakes were last looked at (their last-seen clock less the picks' clock, low
//! byte) `effects::pick_shapes` and `effects::expire_windscreen_flakes`, the picks' clock on one
//! (low byte); then the last-seen clock set to it.
//!
//! Caller: `effects::windscreen_flakes_frame`. The calls are the plain functions.
//! Refuses
//! `pick_shapes`' refusals (all it may write put back first).

use crate::effects::expire_windscreen_flakes::expire_windscreen_flakes;
use crate::effects::mask_store::Masks;
use crate::effects::pick_shapes::pick_shapes;
use crate::effects::windscreen_drop_vars::WindscreenDrops;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;

// Converted: `rc2k_core::effects::steps::{steps, next_step}`
pub fn windscreen_flakes_step(
    masks: &Masks,
    drops: &WindscreenDrops,
    fl: &WindscreenFlakes,
) -> Result<(), Refused> {
    use rc2k_core::effects::steps;
    {
        drops.hex.windscreen_flakes_cleared.set(0);
        let clock: u32 = drops.state.clock.get();
        let n = steps::steps(drops.picks.index.get(), clock);
        drops.picks.index.set(clock);
        for _ in 0..n {
            pick_shapes(masks, drops)?;
            expire_windscreen_flakes(masks, drops, fl);
            drops
                .picks
                .index
                .set(steps::next_step(drops.picks.index.get()));
        }
        drops.state.clock.set_bits(drops.picks.index.get());
    }
    Ok(())
}
