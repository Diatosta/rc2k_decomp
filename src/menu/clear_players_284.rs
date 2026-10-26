//! `menu::clear_players_284` (0x4A9679): every player's `field_284` set to -1.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The four players of `g_players`. Called at the start of each kind of event
//! (`sub_482382` and seven more). Self-test: the game's players, theirs and the test's
//! values, put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::menu::player::{PLAYERS, Player, player_at};
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

pub fn clear_players_284() -> Result<(), Refused> {
    for n in 0..PLAYERS {
        let p = player_at(n);
        if p == 0 {
            return Err(crate::errors::BadInput("a player missing from g_players").into());
        }
    }
    for n in 0..PLAYERS {
        unsafe { (*(player_at(n) as *mut Player)).field_284 = -1 };
    }
    Ok(())
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    clear_players_284()?;
    Ok(Left::nothing())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
