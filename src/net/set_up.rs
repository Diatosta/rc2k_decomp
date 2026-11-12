//! `net::set_up` (0x41848E): what the network game needs before anything else: the
//! receiving thread's event and the mutex (and in the original whether a lobby launched the
//! game).
//!
//! With `g_noticeKind` (a byte) 2, `most_players` (signed) held to 4 at most. An event
//! (auto reset, not signalled: `threading::create_event`) to `event`, none: fatal error
//! 0x80168501; a mutex (`threading::create_mutex`) to `mutex`, none: fatal error
//! 0x80168502. Then DirectPlay's lobby asked (`CoCreateInstance`) whether the game was
//! launched by one ("NET: Lobby launch detected", `g_lobbyLaunched` 1); left out with the
//! lobbies (`net::open_lobby`).
//! One caller, `start`.

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
pub fn set_up(net: &Net) -> Result<(), Refused> {
    let vars = &net.vars;
    if notice_kind() == 2 && net.setup().most_players.get() as i32 > 4 {
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
