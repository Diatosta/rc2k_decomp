//! `net::sessions_frame` (0x41790E): the frame of `net::choose_session`, as
//! `net::connections_frame`'s: the title string 0x150 (0x152 with no session listed), the
//! sessions' names (`net::session_entry`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Called by `screens::run_frame` for `net::choose_session`. Its calls through `part`.
//! Shadow mode runs the original alone (it draws). Self-test: as `net::connections_frame`'s.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::connections_frame::list_frame;
use crate::net::lobby::{Part, SESSIONS, g, lobby_entry};

const SESSION_ENTRY: u32 = 0x418553; // net::session_entry

/// The Rust version: the registers it leaves (the entry's).
pub fn sessions_frame(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::net::screens::sessions_title`.
    let title = rc2k_core::net::screens::sessions_title(g(SESSIONS));
    list_frame(entry, title, SESSIONS, SESSION_ENTRY, 0x51E744, part)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, sessions_frame)
}
