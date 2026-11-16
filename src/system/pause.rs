//! The screens' fixed pauses: the window's events go on during them, so a close is seen at
//! once, and where the pause only gives time to read, a key (or a pad's button) ends it.

use crate::owners::Owners;
use rc2k_core::system::time::{time_up, wait_end};

/// How often a pause looks at the window's events, in milliseconds.
const LOOK_MS: u32 = 10;

/// What a key does to a pause.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Key {
    /// Ends it and is taken.
    Taken,
    /// Ends it and stays the pending key.
    Kept,
    /// Nothing.
    Ignored,
}

/// Waits `milliseconds`, the window's events pumped meanwhile; the game's quitting ends the
/// wait, and a key as `key` says.
fn wait(o: Owners<'static>, milliseconds: u32, key: Key) {
    use crate::input::take_key::{pending_key, take_key};
    use crate::system::tick_count::tick_count;
    use std::sync::atomic::Ordering;
    let clock = &o.system.session.picture_clock;
    let end = wait_end(tick_count(clock), milliseconds);
    loop {
        crate::platform::pump_messages_now(o);
        let ended = match key {
            Key::Taken => take_key(o.input) != 0,
            Key::Kept => pending_key(o.input).load(Ordering::SeqCst) != 0,
            Key::Ignored => false,
        };
        let now = tick_count(clock);
        if ended || o.system.session.flags.quitting.is_set() || time_up(now, end) {
            return;
        }
        crate::system::sleep::sleep(end.wrapping_sub(now).min(LOOK_MS));
    }
}

/// A pause of `milliseconds` before another screen, which a key ends early. The key is
/// taken: the screen that follows does not see it.
pub fn pause(o: Owners<'static>, milliseconds: u32) {
    wait(o, milliseconds, Key::Taken);
}

/// A pause of `milliseconds` inside a menu, which a key ends early. The key stays the
/// pending one: the menu takes it as it would have after the whole pause.
pub fn pause_keeping_key(o: Owners<'static>, milliseconds: u32) {
    wait(o, milliseconds, Key::Kept);
}

/// A pause of `milliseconds` that keys do not end (the time before a screen takes keys, so
/// that a key still held does not answer it); a key pressed meanwhile stays the pending one.
pub fn hold(o: Owners<'static>, milliseconds: u32) {
    wait(o, milliseconds, Key::Ignored);
}
