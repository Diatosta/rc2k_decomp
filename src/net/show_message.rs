//! `net::show_message` (0x41745C): the message shown (`net::message_frame` through
//! `screens::run_frame`), a quarter of a second's sleep, the pending key dropped
//! (`input::take_key`), then frames until a key is pressed.
//!
//! Takes `message`.
//! Caller: `net::start_link`. The frame's message goes to `net::message_frame`
//! (through `screens::run_frame`).

use crate::errors::Refused;
use crate::net::lobby::run_frame;

/// The message shown until a key is pressed.
pub fn show_message(o: crate::owners::Owners<'static>, message: u32) -> Result<(), Refused> {
    let input = o.input;
    run_frame(o, || crate::net::message_frame::message_frame(o, message))?;
    crate::system::sleep::sleep(250);
    crate::input::take_key::take_key(input);
    loop {
        run_frame(o, || crate::net::message_frame::message_frame(o, message))?;
        if crate::input::take_key::take_key(input) != 0 {
            break;
        }
    }
    Ok(())
}
