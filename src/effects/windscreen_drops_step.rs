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
use crate::effects::mask_picks::PICKS;
use crate::effects::windscreen_drop_state::WINDSCREEN_DROP_STATE;
use crate::effects::windscreen_drop_vars::HEX;
use crate::errors::Refused;

/// The Rust version (see the header).
// Converted: `rc2k_core::effects::steps::steps`
pub fn windscreen_drops_step() -> Result<(), Refused> {
    {
        let clock: u32 = WINDSCREEN_DROP_STATE.clock.bits();
        let steps = rc2k_core::effects::steps::steps(PICKS.index.get(), clock);
        PICKS.index.set(clock);
        HEX.steps.set(steps);
        expire_windscreen_drops()?;
        let latest = PICKS.index.get().wrapping_add(HEX.steps.get());
        PICKS.index.set(latest & 0xFF);
        WINDSCREEN_DROP_STATE.clock.set_bits(latest & 0xFF);
    }
    Ok(())
}
