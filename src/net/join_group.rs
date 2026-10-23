//! `net::join_group` (0x419354): the session's groups listed (`EnumGroups` with
//! `net::group_found`, which finds the host's "susanivanovafanclub" into `g_netSendTo`), and
//! this machine's player added to that group (`AddPlayerToGroup`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if either failed, else the flags
//!      of `test eax, eax`
//! Callers: `net::join_session`, `net::open_lobby`. DirectPlay's calls from Rust
//! (`net::dplay`), the callback the game's address (its hook). Shadow mode runs the
//! original alone. Self-test: a copy whose DirectPlay is a test object, against the Rust
//! version: joined, either failing.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::create_group::join;
use crate::net::dplay::{ENUM_GROUPS, call};
use crate::net::lobby::{DPLAY, Part, lobby_entry, test_eax};

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn join_group(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = unsafe {
        call(
            DPLAY,
            ENUM_GROUPS,
            &[
                0,
                crate::net::group_found::callback as *const () as usize as u32,
                0,
                0,
            ],
        )
    };
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
    lobby_entry(r, join_group)
}
