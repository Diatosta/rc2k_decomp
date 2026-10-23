//! `race::add_surface_share` (0x445F7B): a surface's share of the stage, added to.
//!
//! In:  al = the surface (low four bits)
//! Out: with a weight, eax & 0xF; the rest preserved
//!
//! See `race/best_line.rs`. Eight call sites in `sub_445E30` (whose Rust version keeps the
//! exact x87 `add_surface_share`: it rounds the shares to whole numbers). The Rust version
//! is `add_surface_share_plain`. Shadow mode: by state, the shares within `FLOAT`.
//! Self-test, at the test precisions: the original and the Rust version on the game's
//! variables, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_read};
use crate::race::best_line::*;
use crate::screens::harness::Left;
use crate::service::floats::floats_entry;

fn rust(r: &Regs) -> Result<Left, Refused> {
    let weighted = unsafe { game_read::<u32>(WEIGHT_RVA) } != 0;
    // Converted: `rc2k_core::race::best_line::surface_share`.
    let g = unsafe { add_surface_share_plain(r) };
    Ok(if weighted {
        Left::nothing().with(EAX, g.eax)
    } else {
        Left::nothing()
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    floats_entry(r, &rust)
}
