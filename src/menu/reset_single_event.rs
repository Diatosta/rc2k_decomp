//! `menu::reset_single_event` (0x4AAA2F): `dword_755254` / `dword_755950` 0; every
//! driver's +0x10 / +0xC / +0x20 0 and +0x24 1; `dword_75592C` 1, `dword_755978` 0; the
//! four players' +0x8C / +0x84 0 and +0xD0 1 (a reading: a single event's start).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4AA4D7` (twice). Refuses a missing player or a drivers' table without its
//! end entry, before anything. Self-test: the original and the Rust version on the game's
//! tables; put back.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.
use crate::errors::{BadInput, Refused};
use crate::framework::{game_write, rva};
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn reset_single_event() -> Result<(), Refused> {
    if (0..4).any(|n| player(n) == 0) {
        return Err(BadInput("a missing player").into());
    }
    let mut drivers = driver_table()?;
    unsafe {
        game_write(rva(0x755254), 0u32);
        game_write(rva(0x755950), 0u32);
    }
    // Converted: `rc2k_core::menu::event::{DRIVER_BEGIN, PLAYER_BEGIN}`.
    use rc2k_core::menu::event::{DRIVER_BEGIN, PLAYER_BEGIN};
    drivers.store_all(&DRIVER_BEGIN);
    unsafe {
        game_write(rva(0x75592C), 1u32);
        game_write(rva(0x755978), 0u32);
    }
    for n in (0..4).rev() {
        put_all(player(n), &PLAYER_BEGIN);
    }
    Ok(())
}
