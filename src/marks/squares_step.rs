//! `marks::squares_step` (0x4C0654): `dword_81623A` 0; for each step since the squares were
//! last looked at (`dword_814714` less `dword_84460C`, low byte; `dword_84460C` the clock)
//! `marks::pick_shapes` and `marks::expire_squares`, `dword_84460C` on one (low byte); then
//! `dword_814714` it.
//!
//! In:  nothing
//! Out: nothing (no caller reads a register it leaves)
//!
//! Caller: `sub_4BF60F` (0x4BF809). The calls are their Rust versions. Refuses
//! `pick_shapes`' refusals (all it may write put back first). Self-test: the original and
//! the Rust version on the game's items (as `expire_squares`'), clocks by lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::marks::expire_squares::expire_squares;
use crate::marks::pick_shapes::pick_shapes;

const CLEARED_RVA: usize = rva(0x81623A); // dword_81623A
const CLOCK_RVA: usize = rva(0x814714); // dword_814714
const LATEST_RVA: usize = rva(0x84460C); // dword_84460C

/// The Rust version (see the header).
// Converted: `rc2k_core::marks::steps::{steps, next_step}`
pub unsafe fn squares_step() -> Result<(), Refused> {
    use rc2k_core::marks::steps;
    unsafe {
        game_write(CLEARED_RVA, 0u32);
        let clock: u32 = game_read(CLOCK_RVA);
        let n = steps::steps(game_read(LATEST_RVA), clock);
        game_write(LATEST_RVA, clock);
        for _ in 0..n {
            pick_shapes()?;
            expire_squares();
            game_write(LATEST_RVA, steps::next_step(game_read(LATEST_RVA)));
        }
        game_write(CLOCK_RVA, game_read::<u32>(LATEST_RVA));
    }
    Ok(())
}
