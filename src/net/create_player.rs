//! `net::create_player` (0x4190FB): "NET: Create player" and the player's name
//! (`byte_52AB70`) logged; a DPNAME made (`dword_5AB29C`: zeroed, its size 0x10, the short
//! name) and this machine's DirectPlay player made with it (`CreatePlayer` into
//! `g_netPlayerId`, the receiving thread's event `g_netEvent`, the player's record
//! `byte_52AD64` as its data, 0x10 bytes, flags 0).
//!
//! In:  nothing
//! Out: true when made (the original: carry clear, the flags of `test eax, eax`; every
//!      register kept), false for the carry
//! Callers: `net::host_session`, `net::join_session`, `net::open_lobby`.

use crate::errors::Refused;
use crate::net::lobby::{log, zero};
use crate::net::store::NET;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

/// A DPNAME at `dword_5AB29C` made: zeroed, its size, the short name.
pub fn dp_name(short_name: u32) -> Result<(), Refused> {
    zero(NET.dp_name.addr(), 0x10)?;
    NET.dp_name.size.set(0x10);
    NET.dp_name.short_name.set(short_name);
    Ok(())
}

/// The Rust version.
pub fn create_player() -> Result<bool, Refused> {
    log(VIEWER_PANEL.text_addr(texts::NET_CREATE_PLAYER), false, 2);
    log(NET.player_name.addr(), true, 2);
    dp_name(NET.player_name.addr())?;
    let result = {
        crate::net::dp_objects::create_player(
            NET.dplay.get(),
            NET.player_id.addr(),
            NET.dp_name.addr(),
            NET.event.get(),
            NET.own.addr(),
            0x10,
            0,
        )
    };
    Ok(result == 0)
}
