//! `race::seal_block` (0x402735): the block (`dword_4E0534` / `38`) enciphered with the
//! key (`dword_4E052C` / `30`) by the game's DES-like tables into `value` (0x4E053C) and
//! `dword_4E0540`, its sixteen subkeys left at `dword_4E04AC` ..; `pusha` / `popa`.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `race/save_seal.rs` (with `race::permute_bits`). Callers: `sub_40264D` and
//! `sub_41F0A3` (each `mov`s `value` after; no flags read). Shadow mode: the Rust version,
//! the subkeys and result put back, the original's, compared. Self-test: keys and blocks
//! by lot (0 and all ones among them), the game's put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::race::save_seal::*;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { seal_block() };
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
