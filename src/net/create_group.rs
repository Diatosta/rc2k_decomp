//! `net::create_group` (0x4192EA): the host's group made, which every player joins and every
//! message goes to: a DPNAME (`dword_5AB29C`) named "susanivanovafanclub" (`0x5AB25C`),
//! `CreateGroup` into `g_netSendTo` (no data, flags 0), and this machine's player added to
//! it (`AddPlayerToGroup`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if either failed, else the flags
//!      of `test eax, eax`
//! Callers: `net::host_session`, `net::open_lobby`. Its call through `part`; DirectPlay's
//! from Rust (`net::dplay`). Shadow mode runs the original alone. Self-test: a copy whose
//! call goes to a stand-in and whose DirectPlay is a test object, against the Rust version:
//! made, either failing.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::create_player::dp_name;
use crate::net::dplay::{ADD_PLAYER_TO_GROUP, CREATE_GROUP, call};
use crate::net::lobby::{DP_NAME, DPLAY, PLAYER_ID, Part, SEND_TO, a, g, lobby_entry, test_eax};

pub const GROUP_NAME: usize = 0x5AB25C; // "susanivanovafanclub"

/// `AddPlayerToGroup(g_netSendTo, g_netPlayerId)`, then the flags; shared with
/// `net::join_group`.
pub fn join(r: &mut Regs) {
    r.eax = unsafe { call(DPLAY, ADD_PLAYER_TO_GROUP, &[g(SEND_TO), g(PLAYER_ID)]) };
    test_eax(r);
    if r.eax != 0 {
        r.set_carry(true);
    }
}

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn create_group(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    dp_name(&r, a(GROUP_NAME), part)?;
    r.eax = unsafe { call(DPLAY, CREATE_GROUP, &[a(SEND_TO), a(DP_NAME), 0, 0, 0]) };
    test_eax(&mut r);
    if r.eax != 0 {
        r.set_carry(true);
    } else {
        join(&mut r);
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, create_group)
}
