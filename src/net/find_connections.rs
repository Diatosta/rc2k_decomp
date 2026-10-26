//! `net::find_connections` (0x41867C): "NET: Find connections"; unless the receiving thread
//! has ended (`g_netReceiveDone` 1): the lobby object made (`net::open_lobby`); unless a
//! lobby launched the game (its connection is made there): DirectPlay's object made
//! (`net::create_dplay`), the lists' memory taken (`net::allocate_lists`), and the
//! connections listed (`EnumConnections` for the game's GUID `dword_5AB300` with
//! `net::connection_found`, flags DPCONNECTION_DIRECTPLAY).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); carry set if it failed or nothing was
//!      listed (the flags of `cmp dword_52AC34, 1`); after a lobby's launch the flags of
//!      `cmp g_lobbyLaunched, 1`
//! Caller: `net::find_connections_regs`. Its calls through `part`; `EnumConnections` from
//! Rust (`net::dplay`), the callback the game's address (its hook). Shadow mode runs the
//! original alone. Self-test: a copy whose calls go to stand-ins and whose DirectPlay is a
//! test object, against the Rust version, on every way out.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::dplay::{ENUM_CONNECTIONS, call};
use crate::net::lobby::{
    APP_GUID, CONNECTIONS, DPLAY, LOBBY_LAUNCHED, LOG_L2_NL, Part, RECEIVE_DONE, a, cmp_flags, g,
    lobby_entry, log_text, p, test_eax,
};

const TEXT: usize = 0x5C2E5F; // "NET: Find connections"
const OPEN_LOBBY: u32 = 0x418E9B; // net::open_lobby
const CREATE_DPLAY: u32 = 0x41861D; // net::create_dplay
const ALLOCATE_LISTS: u32 = 0x418576; // net::allocate_lists
const DIRECTPLAY_CONNECTIONS: u32 = 1; // DPCONNECTION_DIRECTPLAY

/// The Rust version: the registers it leaves (the entry's, the flags).
pub fn find_connections(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    let left = |r: &Regs| Regs {
        eflags: r.eflags,
        ..*entry
    };
    log_text(LOG_L2_NL, a(TEXT), &r, part)?;
    cmp_flags(&mut r, g(RECEIVE_DONE), 1);
    if g(RECEIVE_DONE) == 1 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    part(OPEN_LOBBY, &mut r)?;
    if r.carry() {
        return Ok(left(&r));
    }
    cmp_flags(&mut r, g(LOBBY_LAUNCHED), 1);
    if g(LOBBY_LAUNCHED) == 1 {
        return Ok(left(&r));
    }
    part(CREATE_DPLAY, &mut r)?;
    if r.carry() {
        return Ok(left(&r));
    }
    part(ALLOCATE_LISTS, &mut r)?;
    p(CONNECTIONS, 0);
    r.eax = unsafe {
        call(
            DPLAY,
            ENUM_CONNECTIONS,
            &[
                a(APP_GUID),
                crate::net::connection_found::callback as *const () as usize as u32,
                0,
                DIRECTPLAY_CONNECTIONS,
            ],
        )
    };
    test_eax(&mut r);
    if r.eax != 0 {
        r.set_carry(true);
        return Ok(left(&r));
    }
    cmp_flags(&mut r, g(CONNECTIONS), 1);
    Ok(left(&r))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, find_connections)
}
