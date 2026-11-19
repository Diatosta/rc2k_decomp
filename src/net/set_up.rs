//! What the network game needs before anything else: the receiving thread's event (auto-reset,
//! not set). With the notice kind 2, `most_players` is held to 4 at most.

use crate::net::owner::Net;
use crate::strings::locale::notice_kind;

pub fn set_up(net: &Net, strings: &crate::strings::owner::Strings) {
    let vars = &net.vars;
    if notice_kind(strings) == 2 && net.setup().most_players.get() as i32 > 4 {
        net.setup().most_players.set(4);
    }
    vars.event
        .set(crate::platform::sync::create_event(false, false));
}
