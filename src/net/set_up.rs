//! What the network game needs before anything else: the receiving thread's event and the
//! mutex (fatal errors 0x80168501 and 0x80168502 when either cannot be made). With the notice
//! kind 2, `most_players` is held to 4 at most.

use crate::errors::{FatalError, Refused};
use crate::net::owner::Net;
use crate::strings::load_language::notice_kind;
use crate::threading::create_event::{create_event, handle_flags};
use crate::threading::create_mutex::create_mutex;

const NO_EVENT: FatalError = FatalError {
    code: 0x8016_8501,
    message: "",
};
const NO_MUTEX: FatalError = FatalError {
    code: 0x8016_8502,
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
    let mutex = create_mutex();
    if handle_flags(mutex).0 {
        return Err(NO_MUTEX.into());
    }
    vars.mutex.set(mutex);
    Ok(())
}
