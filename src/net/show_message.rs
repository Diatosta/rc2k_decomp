//! Shows a message, waits a quarter of a second (keys do not end it), drops the pending key, then shows frames
//! until a key is pressed or the game is quitting.

use crate::errors::Refused;
use crate::net::lobby::run_frame;

/// The message shown until a key is pressed.
pub fn show_message(o: crate::owners::Owners<'static>, message: u32) -> Result<(), Refused> {
    let input = o.input;
    run_frame(o, || crate::net::message_frame::message_frame(o, message))?;
    crate::system::pause::hold(o, 250);
    crate::input::take_key::take_key(input);
    loop {
        run_frame(o, || crate::net::message_frame::message_frame(o, message))?;
        // A closed window ends the message as a key does: no frame is shown and no key
        // comes once the game is quitting.
        if crate::input::take_key::take_key(input) != 0 || o.system.session.flags.quitting.is_set()
        {
            break;
        }
    }
    Ok(())
}
