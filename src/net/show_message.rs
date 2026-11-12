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
pub fn show_message(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    message: u32,
    gfx: &'static crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    run_frame(system, || {
        crate::net::message_frame::message_frame(so, message, gfx, system)
    })?;
    crate::system::sleep::sleep(250);
    crate::input::take_key::take_key();
    loop {
        run_frame(system, || {
            crate::net::message_frame::message_frame(so, message, gfx, system)
        })?;
        if crate::input::take_key::take_key() != 0 {
            break;
        }
    }
    Ok(())
}
