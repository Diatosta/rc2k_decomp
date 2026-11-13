//! Closes the open DirectPlay session, if any.

use crate::net::store::NetShared;
pub fn close_session(vars: &NetShared) {
    if vars.session_open.get() == 1 {
        vars.session_open.set(0);
        crate::net::dp_objects::close(vars.dplay.get());
    }
}
