//! Closes the open DirectPlay session, if any.

use crate::net::link::Link;
pub(super) fn close_session(link: &Link) {
    link.close_session();
}
