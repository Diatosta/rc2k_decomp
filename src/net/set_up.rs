//! `net::set_up` (0x41848E): what the network game needs before anything else: the
//! receiving thread's event and the mutex (and in the original whether a lobby launched the
//! game).
//!
//! With `g_noticeKind` (a byte) 2, `dword_52ABD0` (signed) held to 4 at most. An event (auto reset,
//! not signalled: `threading::create_event` with eax 0, ebx 0) to `dword_52AB68`, none:
//! fatal error 0x80168501; a mutex (`threading::create_mutex`) to `dword_52AB6C`, none:
//! fatal error 0x80168502. Then `sub_418E23`: DirectPlay's lobby asked
//! (`CoCreateInstance`) whether the game was launched by one ("NET: Lobby launch
//! detected", `dword_5AB290` 1); left out with the lobbies (`net::open_lobby`).
//! One caller, `start` (0x4010A2).

use crate::errors::{FatalError, Refused};
use crate::net::owner::net;
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

/// The Rust version.
pub fn set_up() -> Result<(), Refused> {
    let vars = &net().vars;
    if notice_kind() == 2 && vars.most_players.get() as i32 > 4 {
        vars.most_players.set(4);
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
