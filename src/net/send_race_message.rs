//! `net::send_race_message` (0x46449E): `call net::send_message; retn`.
//!
//! In:  ebx = the bytes, ecx = how many
//! Out: as `net::send_message`
//!
//! See `net/mod.rs`. Three callers: `net::send_inputs`, `net::send_position`,
//! `net::send_extra`. Its call through `part`. A call in its first five bytes: private copy
//! (6 bytes, the call relocated), manual patch. Shadow mode runs the original alone.
//! Self-test: a copy whose call goes to a noting stand-in, against the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::{Part, SEND_MESSAGE, net_entry};

/// The Rust version.
pub fn send_race_message(r: &mut Regs, part: Part) -> Result<(), Refused> {
    part(SEND_MESSAGE, r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, send_race_message)
}
