//! Waits up to the given milliseconds for a key: every 10 ms
//! (`clock::sleep`) a key taken (`input::take_key`), the clock (`clock::tick_count`)
//! read until it reaches the start plus that (a signed difference). Gives the key, 0 when the
//! time ran out.

use rc2k_core::system::time::{time_up, wait_end};

/// The key, 0 when the time ran out. Its one caller
/// (`menu::championship_welcome`) reads only the key and whether there was one.
pub fn wait_or_key(o: crate::owners::Owners<'static>, milliseconds: u32) -> u8 {
    let (system, input) = (o.system, o.input);
    use crate::clock::tick_count::tick_count;
    let end = wait_end(tick_count(&system.session.picture_clock), milliseconds);
    loop {
        crate::clock::sleep::sleep(10);
        // The window's events are pumped here: the window waits on this thread.
        crate::shell::frame::pump_messages_now(o);
        let key = crate::input::take_key::take_key(input);
        if key != 0 {
            return key;
        }
        if system.session.flags.quitting.is_set()
            || time_up(tick_count(&system.session.picture_clock), end)
        {
            return 0;
        }
    }
}
