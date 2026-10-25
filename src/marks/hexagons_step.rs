//! `marks::hexagons_step` (0x4BCAD0): the steps since the hexagons were last looked at
//! (`dword_814714` less `dword_84460C`, low byte) into `dword_81622E`, `dword_84460C` the
//! clock `dword_814714`; `marks::expire_hexagons`; then `dword_84460C` moved on by those
//! steps (low byte) and back into `dword_814714`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BAC87` (0x4BAF7D). The call is `expire_hexagons`' Rust version.

use crate::marks::expire_hexagons::expire_hexagons;
use crate::marks::hexagon_state::HEXAGON_STATE;
use crate::marks::hexagon_vars::HEX;
use crate::marks::mask_picks::PICKS;

/// The Rust version (see the header).
// Converted: `rc2k_core::marks::steps::steps`
pub unsafe fn hexagons_step() {
    unsafe {
        let clock: u32 = HEXAGON_STATE.clock.bits();
        let steps = rc2k_core::marks::steps::steps(PICKS.index.get(), clock);
        PICKS.index.set(clock);
        HEX.steps.set(steps);
        expire_hexagons();
        let latest = PICKS.index.get().wrapping_add(HEX.steps.get());
        PICKS.index.set(latest & 0xFF);
        HEXAGON_STATE.clock.set_bits(latest & 0xFF);
    }
}
