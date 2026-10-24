//! `net::create_player` (0x4190FB): "NET: Create player" and the player's name
//! (`byte_52AB70`) logged; a DPNAME made (`dword_5AB29C`: zeroed, its size 0x10, the short
//! name) and this machine's DirectPlay player made with it (`CreatePlayer` into
//! `g_netPlayerId`, the receiving thread's event `g_netEvent`, the player's record
//! `byte_52AD64` as its data, 0x10 bytes, flags 0).
//!
//! In:  nothing
//! Out: true when made (the original: carry clear, the flags of `test eax, eax`; every
//!      register kept), false for the carry
//! Callers: `net::host_session`, `net::join_session`, `net::open_lobby`. Its calls through
//! `part`; `CreatePlayer` from Rust (`net::dplay`).

use crate::errors::Refused;
use crate::net::dplay::{CREATE_PLAYER, call};
use crate::net::lobby::{DP_NAME, DPLAY, OWN, PLAYER_ID, PLAYER_NAME, a, g, log, p, zero};

const TEXT: usize = 0x5C2F6E; // "NET: Create player"
const EVENT: usize = 0x52AB68; // g_netEvent

/// A DPNAME at `dword_5AB29C` made: zeroed, its size, the short name.
pub fn dp_name(short_name: u32) -> Result<(), Refused> {
    zero(a(DP_NAME), 0x10)?;
    p(DP_NAME, 0x10);
    p(DP_NAME + 8, short_name);
    Ok(())
}

/// The Rust version.
pub fn create_player() -> Result<bool, Refused> {
    log(a(TEXT), false, 2);
    log(a(PLAYER_NAME), true, 2);
    dp_name(a(PLAYER_NAME))?;
    let result = unsafe {
        call(
            DPLAY,
            CREATE_PLAYER,
            &[a(PLAYER_ID), a(DP_NAME), g(EVENT), a(OWN), 0x10, 0],
        )
    };
    Ok(result == 0)
}
