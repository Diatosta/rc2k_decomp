//! `net::show_message` (0x41745C): message eax shown (`net::message_frame` through
//! `screens::run_frame`), a quarter of a second's sleep, the pending key dropped
//! (`input::take_key`), then frames until a key is pressed.
//!
//! Takes `message`.
//! Caller: `net::start_link`. The frame's message goes to `net::message_frame` in esi
//! (through `screens::run_frame`).

use crate::errors::Refused;
use crate::net::lobby::run_frame;

/// The Rust version: the message shown until a key is pressed.
pub fn show_message(message: u32) -> Result<(), Refused> {
    run_frame(|| crate::net::message_frame::message_frame(message))?;
    crate::system::sleep::sleep(250);
    crate::input::take_key::take_key();
    loop {
        run_frame(|| crate::net::message_frame::message_frame(message))?;
        if crate::input::take_key::take_key() != 0 {
            break;
        }
    }
    Ok(())
}
