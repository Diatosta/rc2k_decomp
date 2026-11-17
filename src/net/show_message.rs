//! Shows a message, waits a quarter of a second (keys do not end it), drops the pending key, then shows frames
//! until a key is pressed or the game is quitting.

use crate::errors::Refused;
use crate::net::lobby::run_frame;

/// The message shown until a key is pressed.
pub fn show_message(o: crate::owners::Owners<'static>, message: u32) -> Result<(), Refused> {
    show_frames(o, || crate::net::message_frame::message_frame(o, message))
}

/// The message that the port is in use by another program, with the port's number in it.
pub fn show_port_in_use(o: crate::owners::Owners<'static>, port: u16) -> Result<(), Refused> {
    let template = crate::strings::append_string_by_id::own_text(
        o.strings,
        rc2k_core::strings::own::id::PORT_IN_USE,
    );
    let text = String::from_utf8_lossy(&template)
        .replace("{}", &port.to_string())
        .into_bytes();
    show_frames(o, || crate::net::message_frame::text_frame(o, &text))
}

/// Shows `frame` until a key is pressed.
fn show_frames(
    o: crate::owners::Owners<'static>,
    mut frame: impl FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    let input = o.input;
    run_frame(o, &mut frame)?;
    crate::system::pause::hold(o, 250);
    crate::input::take_key::take_key(input);
    loop {
        run_frame(o, &mut frame)?;
        // A closed window ends the message as a key does: no frame is shown and no key
        // comes once the game is quitting.
        if crate::input::take_key::take_key(input) != 0 || o.system.session.flags.quitting.is_set()
        {
            break;
        }
    }
    Ok(())
}
