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
    // The template is already in the game's text encoding (Windows-1252): the port goes in as
    // bytes, never through a UTF-8 string (that turned every accent into a broken character).
    let text = fill_number(&template, port);
    show_frames(o, || crate::net::message_frame::text_frame(o, &text))
}

/// A message of our own (`rc2k_core::strings::own::id`) shown until a key is pressed.
pub fn show_own_text(o: crate::owners::Owners<'static>, id: u32) -> Result<(), Refused> {
    let text = crate::strings::append_string_by_id::own_text(o.strings, id);
    show_frames(o, || crate::net::message_frame::text_frame(o, &text))
}

/// `template` with its first `{}` replaced by the number's digits; the other bytes untouched.
fn fill_number(template: &[u8], number: u16) -> Vec<u8> {
    let mut out = Vec::with_capacity(template.len() + 5);
    match template.windows(2).position(|w| w == b"{}") {
        Some(at) => {
            out.extend_from_slice(&template[..at]);
            out.extend_from_slice(number.to_string().as_bytes());
            out.extend_from_slice(&template[at + 2..]);
        }
        None => out.extend_from_slice(template),
    }
    out
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

#[cfg(test)]
mod tests {
    use super::fill_number;

    #[test]
    fn the_number_goes_in_and_accents_stay() {
        // "está" in Windows-1252
        let template = b"A porta {} est\xE1 a ser usada.";
        assert_eq!(
            fill_number(template, 47624),
            b"A porta 47624 est\xE1 a ser usada."
        );
        assert_eq!(fill_number(b"none", 1), b"none");
    }
}
