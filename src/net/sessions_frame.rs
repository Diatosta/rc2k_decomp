//! `net::sessions_frame` (0x41790E): the frame of `net::choose_session`, as
//! `net::connections_frame`'s: the title string 0x150 (0x152 with no session listed), the
//! sessions' names (`net::session_entry`).
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`)
//! Called by `screens::run_frame` for `net::choose_session` (`routes.rs`).
//! Called by `screens::run_frame` for `net::choose_session`.

use crate::errors::Refused;
use crate::net::connections_frame::list_frame;

/// The Rust version.
pub fn sessions_frame() -> Result<(), Refused> {
    // Converted: `rc2k_core::net::screens::sessions_title`.
    let title =
        rc2k_core::net::screens::sessions_title(crate::net::owner::net().vars.sessions.get());
    list_frame(title, &crate::net::owner::net().vars.sessions, |n| {
        crate::net::tables::with_session(n, |s| s.name)
    })
}
