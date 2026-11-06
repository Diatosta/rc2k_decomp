//! `effects::windscreen_flakes_step` (0x4C0654): `dword_81623A` 0; for each step since the windscreen flakes were
//! last looked at (`dword_814714` less `dword_84460C`, low byte; `dword_84460C` the clock)
//! `effects::pick_shapes` and `effects::expire_windscreen_flakes`, `dword_84460C` on one (low byte); then
//! `dword_814714` it.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4BF60F` (0x4BF809). The calls are their Rust versions. Refuses
//! `pick_shapes`' refusals (all it may write put back first).

use crate::effects::expire_windscreen_flakes::expire_windscreen_flakes;
use crate::effects::mask_picks::PICKS;
use crate::effects::pick_shapes::pick_shapes;
use crate::effects::windscreen_drop_state::WINDSCREEN_DROP_STATE;
use crate::effects::windscreen_drop_vars::HEX;
use crate::effects::windscreen_flakes_store::WindscreenFlakes;
use crate::errors::Refused;

/// The Rust version (see the header).
// Converted: `rc2k_core::effects::steps::{steps, next_step}`
pub fn windscreen_flakes_step(fl: &WindscreenFlakes) -> Result<(), Refused> {
    use rc2k_core::effects::steps;
    {
        HEX.windscreen_flakes_cleared.set(0);
        let clock: u32 = WINDSCREEN_DROP_STATE.clock.bits();
        let n = steps::steps(PICKS.index.get(), clock);
        PICKS.index.set(clock);
        for _ in 0..n {
            pick_shapes()?;
            expire_windscreen_flakes(fl);
            PICKS.index.set(steps::next_step(PICKS.index.get()));
        }
        WINDSCREEN_DROP_STATE.clock.set_bits(PICKS.index.get());
    }
    Ok(())
}
