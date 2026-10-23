//! `net::dplay_send` (0x4196FF): DirectPlay's `Send` (`IDirectPlay4A` +0x68) from this
//! machine's player (`g_netPlayerId`) through `g_dplay`.
//!
//! In:  eax = to whom (a player or the group), ebx = the bytes, ecx = how many, edx = the
//!      flags (DPSEND_*)
//! Out: every register kept (eax, ecx and edx pushed and popped); success: the flags of
//!      `test eax, eax` on the result 0 (zero set, carry clear); failure: those of the
//!      result, then carry set
//! Six callers (`net::send_message`, `net::send_chat`, `net::race_wait`, `net::lobby_loop`,
//! `net::take_messages` twice). `Send` from Rust (`net::dplay`, section 15). Refuses nothing
//! (no check of `g_dplay`, as the original). Shadow mode runs the original alone (the
//! message is sent). Self-test: a copy whose two variables are the test's and whose
//! DirectPlay is a test object noting its arguments, against the Rust version: sent, failed.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::dplay::{SEND, method};
use crate::net::lobby::test_eax;
use crate::net::message_vars::{MessageVars, get};
use crate::net::{Part, net_entry};

/// The Rust version on the variables at `v`.
pub fn dplay_send_with(r: &mut Regs, v: &MessageVars) {
    let result = unsafe {
        method(
            get(v.dplay),
            SEND,
            &[get(v.player_id), r.eax, r.edx, r.ebx, r.ecx],
        )
    };
    let mut f = Regs { eax: result, ..*r };
    test_eax(&mut f);
    if result != 0 {
        f.set_carry(true);
    }
    r.eflags = f.eflags;
}

/// The Rust version.
pub fn dplay_send(r: &mut Regs, _part: Part) -> Result<(), Refused> {
    dplay_send_with(r, &MessageVars::game());
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, dplay_send)
}
