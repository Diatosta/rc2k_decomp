//! Shows a message, sleeps a quarter of a second, drops the pending key, then shows frames
//! until a key is pressed.

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
