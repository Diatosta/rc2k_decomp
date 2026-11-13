//! `system::wait_or_key` (0x407F69): waits up to the given milliseconds for a key: every 10 ms
//! (`system::sleep`) a key taken (`input::take_key`), the clock (`system::tick_count`)
//! read until it reaches the start plus that (a signed difference).
//!
//! Takes the milliseconds. Returns the key (0 when the time ran out).
//!
//! Caller: `menu::championship_welcome` (the welcome's speech).

use rc2k_core::system::time::{time_up, wait_end};

/// The key, 0 when the time ran out. Its one caller
/// (`menu::championship_welcome`) reads only the key and whether there was one.
pub fn wait_or_key(o: crate::owners::Owners<'static>, milliseconds: u32) -> u8 {
    let (system, input) = (o.system, o.input);
    use crate::system::tick_count::tick_count;
    // Converted: `rc2k_core::system::time::{wait_end, time_up}`.
    let end = wait_end(tick_count(), milliseconds);
    loop {
        crate::system::sleep::sleep(10);
        // The original's window thread saw to these; here the window waits on this thread.
        crate::platform::pump_messages_now(o);
        let key = crate::input::take_key::take_key(input);
        if key != 0 {
            return key;
        }
        if system.session.flags.quitting.is_set() || time_up(tick_count(), end) {
            return 0;
        }
    }
}
