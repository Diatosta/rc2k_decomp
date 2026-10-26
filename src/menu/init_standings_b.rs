//! `menu::init_standings_b` (0x4A4644): the 200 entries of standings table B emptied (their
//! first five dwords -1).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A44A0`. Refuses a null table, before anything. Self-test: the original and
//! the Rust version on the game's table; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn init_standings_b(entry: &Regs) -> Result<Regs, Refused> {
    let first: u32 = unsafe { game_read(TABLE_B_RVA) };
    if first == 0 {
        return Err(BadInput("no standings table").into());
    }
    // Converted: `rc2k_core::menu::standings::EMPTY_B`.
    use rc2k_core::menu::standings::{EMPTY_B, ROOM};
    for n in 0..ROOM {
        for (k, v) in EMPTY_B.into_iter().enumerate() {
            put(first + B_ENTRY * n + 4 * k as u32, v);
        }
    }
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { init_standings_b(r) }?))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
