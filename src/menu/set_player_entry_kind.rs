//! `menu::set_player_entry_kind` (0x4A1922): the current player's (`dword_75598C`) +0xD0
//! 1 when `dword_75AC80` is 100 or more (signed), else 2.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_48E093`, `sub_4A15E3`. Refuses a current player past 3 or missing, before
//! anything. Self-test: the original and the Rust version on players of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, rva};
use crate::menu::standings::*;

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const SCORE_RVA: usize = rva(0x75AC80); // dword_75AC80
/// The Rust version (see the header): the registers it leaves.
pub unsafe fn set_player_entry_kind() -> Result<(), Refused> {
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    // Converted: `rc2k_core::menu::player::entry_kind`.
    let score: i32 = unsafe { game_read(SCORE_RVA) };
    put(player(n) + 0xD0, rc2k_core::menu::player::entry_kind(score));
    Ok(())
}
