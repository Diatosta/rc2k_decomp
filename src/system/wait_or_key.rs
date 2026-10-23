//! `system::wait_or_key` (0x407F69): waits up to eax milliseconds for a key: every 10 ms
//! (`system::sleep`) a key taken (`input::take_key`), the clock (`system::tick_count`)
//! read until it reaches the start plus eax (a signed difference).
//!
//! In:  eax the milliseconds
//! Out: eax the key (0 when the time ran out), zero = no key, carry clear; ebx kept (its
//!      `push` / `pop`); the rest as the calls leave them
//!
//! Caller: `sub_47F314` (the welcome's speech). The calls through `part`, in the game
//! their Rust versions. A private copy (a call at +3). Ungated (it waits). Shadow mode
//! runs the original alone (it takes keys). Self-test:
//! `menu::scripted::test_loop`, keys and clocks by script.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use rc2k_core::system::time::{time_up, wait_end};

/// The Rust version (see the header): the registers it leaves.
pub fn wait_or_key(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    g.ebx = g.eax;
    crate::system::wait_or_key::game_part(1, &mut g)?;
    // Converted: `rc2k_core::system::time::{wait_end, time_up}`.
    g.ebx = wait_end(g.eax, g.ebx);
    loop {
        g.eax = 10;
        crate::system::wait_or_key::game_part(2, &mut g)?;
        g.eax = 0;
        crate::system::wait_or_key::game_part(3, &mut g)?;
        if g.eflags & 0x40 == 0 {
            break;
        }
        crate::system::wait_or_key::game_part(1, &mut g)?;
        if time_up(g.eax, g.ebx) {
            g.eax = 0;
            break;
        }
    }
    g.ebx = entry.ebx;
    g.set_zero(g.eax == 0);
    g.set_carry(false);
    Ok(g)
}

/// In the game: the Rust versions.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    match place {
        1 => r.eax = crate::system::tick_count::tick_count(),
        2 => crate::system::sleep::sleep(r.eax),
        _ => {
            let key = crate::input::take_key::take_key();
            r.eax = (r.eax & !0xFF) | u32::from(key);
            r.set_zero(key == 0);
        }
    }
    Ok(())
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    // The game's parts never fail.
    if let Ok(left) = wait_or_key(r) {
        *r = Regs { esp: r.esp, ..left };
    }
    Ok(())
}
