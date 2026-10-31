//! `system::wait_or_key` (0x407F69): waits up to eax milliseconds for a key: every 10 ms
//! (`system::sleep`) a key taken (`input::take_key`), the clock (`system::tick_count`)
//! read until it reaches the start plus eax (a signed difference).
//!
//! In:  eax the milliseconds
//! Out: eax the key (0 when the time ran out), zero = no key, carry clear; ebx kept (its
//!      `push` / `pop`); the rest as the calls leave them
//!
//! Caller: `sub_47F314` (the welcome's speech). A private copy (a call at +3). Ungated (it waits). Shadow mode
//! runs the original alone (it takes keys). Self-test:
//! `menu::scripted::test_loop`, keys and clocks by script.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use rc2k_core::system::time::{time_up, wait_end};

/// The Rust version (see the header): the key, 0 when the time ran out. The original
/// leaves it in eax with zero set for none and carry clear, ebx kept; the rest are as the
/// calls leave them, which its one caller (`menu::championship_welcome`) does not read
/// (register liveness, 2026-10-23: after the call ZF and eax).
pub fn wait_or_key(milliseconds: u32) -> u8 {
    use crate::system::tick_count::tick_count;
    // Converted: `rc2k_core::system::time::{wait_end, time_up}`.
    let end = wait_end(tick_count(), milliseconds);
    loop {
        crate::system::sleep::sleep(10);
        // The original's window thread saw to these; here the window waits on this thread.
        crate::platform::pump_messages_now();
        let key = crate::input::take_key::take_key();
        if key != 0 {
            return key;
        }
        if crate::platform::quitting() || time_up(tick_count(), end) {
            return 0;
        }
    }
}
