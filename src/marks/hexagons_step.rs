//! `marks::hexagons_step` (0x4BCAD0): the steps since the hexagons were last looked at
//! (`dword_814714` less `dword_84460C`, low byte) into `dword_81622E`, `dword_84460C` the
//! clock `dword_814714`; `marks::expire_hexagons`; then `dword_84460C` moved on by those
//! steps (low byte) and back into `dword_814714`.
//!
//! In:  nothing
//! Out: nothing (its caller reads no register after it)
//!
//! Caller: `sub_4BAC87` (0x4BAF7D). The call is `expire_hexagons`' Rust version.
//! Self-test: the original and the Rust version on the game's items (as
//! `expire_hexagons`'), clocks by lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::marks::expire_hexagons::{LATEST_RVA, STEPS_RVA, expire_hexagons};
use crate::marks::store::{Slot, game_read, game_write};

const CLOCK_RVA: Slot = Slot::at(0x814714); // dword_814714

/// The Rust version (see the header).
// Converted: `rc2k_core::marks::steps::steps`
pub unsafe fn hexagons_step() {
    unsafe {
        let clock: u32 = game_read(CLOCK_RVA);
        let steps = rc2k_core::marks::steps::steps(game_read(LATEST_RVA), clock);
        game_write(LATEST_RVA, clock);
        game_write(STEPS_RVA, steps);
        expire_hexagons();
        let latest = game_read::<u32>(LATEST_RVA).wrapping_add(game_read(STEPS_RVA));
        game_write(LATEST_RVA, latest & 0xFF);
        game_write(CLOCK_RVA, latest & 0xFF);
    }
}
