//! `menu::rank_best_score_4` (0x4A922F): as `menu::rank_best_score` for group 4 (the current player is read but not used).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A871C`. Refuses a current player past 3 or missing (the original reads it) and a null table, before anything. Self-test: the original and the Rust version on the game's tables, scores by lot; put
//! back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn rank_best_score_4(entry: &Regs) -> Result<Regs, Refused> {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    rank_in_group(4)?;
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { rank_best_score_4(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
