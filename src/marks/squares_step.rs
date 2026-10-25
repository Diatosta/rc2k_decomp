//! `marks::squares_step` (0x4C0654): `dword_81623A` 0; for each step since the squares were
//! last looked at (`dword_814714` less `dword_84460C`, low byte; `dword_84460C` the clock)
//! `marks::pick_shapes` and `marks::expire_squares`, `dword_84460C` on one (low byte); then
//! `dword_814714` it.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4BF60F` (0x4BF809). The calls are their Rust versions. Refuses
//! `pick_shapes`' refusals (all it may write put back first).

use crate::errors::Refused;
use crate::marks::expire_squares::expire_squares;
use crate::marks::hexagon_state::HEXAGON_STATE;
use crate::marks::hexagon_vars::HEX;
use crate::marks::mask_picks::PICKS;
use crate::marks::pick_shapes::pick_shapes;

/// The Rust version (see the header).
// Converted: `rc2k_core::marks::steps::{steps, next_step}`
pub unsafe fn squares_step() -> Result<(), Refused> {
    use rc2k_core::marks::steps;
    unsafe {
        HEX.squares_cleared.set(0);
        let clock: u32 = HEXAGON_STATE.clock.bits();
        let n = steps::steps(PICKS.index.get(), clock);
        PICKS.index.set(clock);
        for _ in 0..n {
            pick_shapes()?;
            expire_squares();
            PICKS.index.set(steps::next_step(PICKS.index.get()));
        }
        HEXAGON_STATE.clock.set_bits(PICKS.index.get());
    }
    Ok(())
}
