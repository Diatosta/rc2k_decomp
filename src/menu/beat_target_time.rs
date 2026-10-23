//! `menu::beat_target_time` (0x4A3C23): `dword_755918` 1 when the current player's
//! (`dword_75598C`) time (+0x88) is at most the current event's target (its +0x57C +
//! 0x14; signed), else 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A3BE0`. Refuses a record past the table, no events table, or a current
//! player past 3 or missing, before anything. Self-test: the original and the Rust version
//! on the game's tables, times by lot; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::menu::standings::*;

const CURRENT_RVA: usize = rva(0x75598C); // dword_75598C
const FLAG_RVA: usize = rva(0x755918); // dword_755918
/// The Rust version (see the header).
pub unsafe fn beat_target_time() -> Result<(), Refused> {
    let event = current_event()?;
    let n: u32 = unsafe { game_read(CURRENT_RVA) };
    if n >= 4 || player(n) == 0 {
        return Err(BadInput("a current player past 3 or missing").into());
    }
    // Converted: `rc2k_core::menu::bests::makes_time`.
    let beaten =
        rc2k_core::menu::bests::makes_time(get(player(n) + 0x88), get(event + 0x57C + 0x14));
    unsafe { game_write(FLAG_RVA, u32::from(beaten)) };
    Ok(())
}
