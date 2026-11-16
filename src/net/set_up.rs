//! What the network game needs before anything else: the receiving thread's event (fatal error
//! 0x80168501 when it cannot be made). With the notice kind 2, `most_players` is held to 4 at
//! most.

use crate::errors::{FatalError, Refused};
use crate::net::owner::Net;
use crate::strings::load_language::notice_kind;
use crate::threading::create_event::{create_event, handle_flags};

const NO_EVENT: FatalError = FatalError {
    code: 0x8016_8501,
    message: "",
};
pub fn set_up(net: &Net, strings: &crate::strings::owner::Strings) -> Result<(), Refused> {
    let vars = &net.vars;
    if notice_kind(strings) == 2 && net.setup().most_players.get() as i32 > 4 {
        net.setup().most_players.set(4);
    }
    let event = create_event(0, 0);
    if handle_flags(event).0 {
        return Err(NO_EVENT.into());
    }
    vars.event.set(event);
    Ok(())
}
