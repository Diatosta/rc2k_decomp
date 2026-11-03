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
//!
//! Since 2026-10-29 the name goes to `net::dp_objects` as text: the DPNAME block
//! (`dword_5AB29C`) is no longer made (nothing else reads it).

use crate::errors::Refused;
use crate::net::lobby::{log, log_bytes};
use crate::net::store::NET;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

/// The Rust version (see the header).
pub fn create_player() -> Result<bool, Refused> {
    log(VIEWER_PANEL.text_ptr(texts::NET_CREATE_PLAYER), false, 2);
    let name = NET.player_name.read();
    log_bytes(&name, true, 2);
    let made = crate::net::dp_objects::create_player(
        NET.dplay.get(),
        &name,
        NET.event.get(),
        &NET.own.bytes(),
    );
    if let Ok(id) = made {
        NET.player_id.set(id);
    }
    Ok(made.is_ok())
}
