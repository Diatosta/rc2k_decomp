//! `net::create_player` (0x4190FB): "NET: Create player" and the player's name
//! (`byte_52AB70`) logged; a DPNAME made (`dword_5AB29C`: zeroed, its size 0x10, the short
//! name) and this machine's DirectPlay player made with it (`CreatePlayer` into
//! `g_netPlayerId`, the receiving thread's event `g_netEvent`, the player's record
//! `byte_52AD64` as its data, 0x10 bytes, flags 0).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if it failed, else the flags of
//!      `test eax, eax`
//! Callers: `net::host_session`, `net::join_session`, `net::open_lobby`. Its calls through
//! `part`; `CreatePlayer` from Rust (`net::dplay`). Shadow mode runs the original alone.
//! Self-test: a copy whose calls go to stand-ins and whose DirectPlay is a test object,
//! against the Rust version: made, failing.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::dplay::{CREATE_PLAYER, call};
use crate::net::lobby::{
    DP_NAME, DPLAY, LOG_L2, LOG_L2_NL, OWN, PLAYER_ID, PLAYER_NAME, Part, ZERO_MEMORY, a, g,
    lobby_entry, log_text, p, test_eax,
};

const TEXT: usize = 0x5C2F6E; // "NET: Create player"
const EVENT: usize = 0x52AB68; // g_netEvent

/// A DPNAME at `dword_5AB29C` made: zeroed, its size, the short name.
pub fn dp_name(r: &Regs, short_name: u32, part: Part) -> Result<(), Refused> {
    let mut c = Regs {
        ebx: a(DP_NAME),
        ecx: 0x10,
        ..*r
    };
    part(ZERO_MEMORY, &mut c)?;
    p(DP_NAME, 0x10);
    p(DP_NAME + 8, short_name);
    Ok(())
}

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn create_player(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    log_text(LOG_L2, a(TEXT), &r, part)?;
    log_text(LOG_L2_NL, a(PLAYER_NAME), &r, part)?;
    dp_name(&r, a(PLAYER_NAME), part)?;
    r.eax = unsafe {
        call(
            DPLAY,
            CREATE_PLAYER,
            &[a(PLAYER_ID), a(DP_NAME), g(EVENT), a(OWN), 0x10, 0],
        )
    };
    test_eax(&mut r);
    if r.eax != 0 {
        r.set_carry(true);
    }
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, create_player)
}
