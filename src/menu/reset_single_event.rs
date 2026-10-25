//! `menu::reset_single_event` (0x4AAA2F): `dword_755254` / `dword_755950` 0; every
//! driver's +0x10 / +0xC / +0x20 0 and +0x24 1; `dword_75592C` 1, `dword_755978` 0; the
//! four players' +0x8C / +0x84 0 and +0xD0 1 (a reading: a single event's start).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4AA4D7` (twice). Refuses a missing player or a drivers' table without its end
//! entry, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::event_store::EVENT_STATE;
use crate::menu::pages_store::MENU_PAGES;
use crate::menu::standings::*;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn reset_single_event() -> Result<(), Refused> {
    if (0..4).any(|n| player(n) == 0) {
        return Err(BadInput("a missing player").into());
    }
    let mut drivers = driver_table()?;

    MENU_PAGES.services.set(0u32);
    EVENT_STATE.stages_done.set(0u32);

    // Converted: `rc2k_core::menu::event::{DRIVER_BEGIN, PLAYER_BEGIN}`.
    use rc2k_core::menu::event::{DRIVER_BEGIN, PLAYER_BEGIN};
    drivers.store_all(&DRIVER_BEGIN);

    EVENT_STATE.event_goes_on.set(1u32);
    EVENT_STATE.event_out.set(0u32);

    for n in (0..4).rev() {
        put_all(player(n), &PLAYER_BEGIN);
    }
    Ok(())
}
