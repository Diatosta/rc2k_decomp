//! `net::open_lobby` (0x418E9B): "NET: Create dplobby", DirectPlay's lobby object made
//! (`CoCreateInstance` of CLSID_DirectPlayLobby `stru_5AB340`, IID_IDirectPlayLobby2A
//! `stru_5AB320`) into `g_dplayLobby`; then, for a game a lobby launched
//! (`g_lobbyLaunched` 1): its connection settings read (`GetConnectionSettings` twice: the
//! size, which must come back DPERR_BUFFERTOOSMALL, into `dword_5AB298`; the settings into
//! memory of the network's list, `dword_5AB294`), the session's flags made 0x44 and the
//! settings put back (`SetConnectionSettings`), the lobby's `Connect` into the object
//! whose address `dword_5AB284` holds, `g_dplay` asked for IDirectPlay4A
//! (`QueryInterface`), the object at `dword_5AB284` released and the variable 0; the
//! player's name from the settings (`strings::copy_printable` to `byte_52AB70`), the player
//! set up (`net::fill_own_player`) and made (`net::create_player`); then the session's group
//! made (`net::create_group`, as the host: state bit 0x10) when the settings say to create
//! the session (flag 2), else joined (`net::join_group`); the players listed
//! (`net::list_players`).
//!
//! In:  nothing
//! Out: true when done (the original: carry clear, the flags of `test eax, eax`; every
//!      register kept), false for the carry
//! `Connect` is given `dword_5AB284`'s value as where to write, and that is 0: it can only
//! fail, so a lobby's launch never gets further (section 16). Past it the original would
//! read through `g_dplay` and the object at `dword_5AB284`: refused while they are null.
//! Caller: `net::find_connections`.

use crate::errors::{BadInput, Refused};
use crate::net::create_dplay::IID_DIRECTPLAY4A;
use crate::net::dplay::{
    CONNECT, GET_CONNECTION_SETTINGS, QUERY_INTERFACE, RELEASE, SET_CONNECTION_SETTINGS, call,
    method,
};
use crate::net::lobby::{
    DPLAY, DPLAY_2, LOBBY, LOBBY_LAUNCHED, OWN, PLAYER_NAME, SETTINGS, SETTINGS_SIZE, a, co_create,
    copy_printable, g, gb, log, net_allocate, p, pb,
};
use crate::net::{create_group, create_player, fill_own_player, join_group, list_players};

const TEXT: usize = 0x5C2F5A; // "NET: Create dplobby"
pub const CLSID_DIRECTPLAY_LOBBY: usize = 0x5AB340; // stru_5AB340
pub const IID_DIRECTPLAY_LOBBY2A: usize = 0x5AB320; // stru_5AB320
const BUFFER_TOO_SMALL: u32 = 0x8877_001E; // DPERR_BUFFERTOOSMALL

fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

/// The Rust version.
pub fn open_lobby() -> Result<bool, Refused> {
    log(a(TEXT), true, 2);
    if co_create(
        a(CLSID_DIRECTPLAY_LOBBY),
        a(IID_DIRECTPLAY_LOBBY2A),
        a(LOBBY),
    ) != 0
    {
        return Ok(false);
    }
    if g(LOBBY_LAUNCHED) != 1 {
        return Ok(true);
    }
    unsafe {
        p(SETTINGS_SIZE, 0);
        if call(LOBBY, GET_CONNECTION_SETTINGS, &[0, 0, a(SETTINGS_SIZE)]) != BUFFER_TOO_SMALL {
            return Ok(false);
        }
        p(SETTINGS, net_allocate(g(SETTINGS_SIZE))?);
        if call(
            LOBBY,
            GET_CONNECTION_SETTINGS,
            &[0, g(SETTINGS), a(SETTINGS_SIZE)],
        ) != 0
        {
            return Ok(false);
        }
        ((get(g(SETTINGS) + 8) + 4) as *mut u32).write_unaligned(0x44);
        if call(LOBBY, SET_CONNECTION_SETTINGS, &[0, 0, g(SETTINGS)]) != 0 {
            return Ok(false);
        }
        if call(LOBBY, CONNECT, &[0, g(DPLAY_2), 0]) != 0 {
            return Ok(false);
        }
        if g(DPLAY) == 0 || g(DPLAY_2) == 0 {
            return Err(BadInput("a lobby's connection with no DirectPlay object").into());
        }
        let queried = call(DPLAY, QUERY_INTERFACE, &[a(IID_DIRECTPLAY4A), a(DPLAY)]);
        method(g(DPLAY_2), RELEASE, &[]);
        p(DPLAY_2, 0);
        if queried != 0 {
            return Ok(false);
        }
        let name = get(get(g(SETTINGS) + 0xC) + 8);
        copy_printable(name, a(PLAYER_NAME), 0x1F)?;
        fill_own_player::fill_own_player(name)?;
        if !create_player::create_player()? {
            return Ok(false);
        }
        let joined = if get(g(SETTINGS) + 4) & 2 != 0 {
            pb(OWN, gb(OWN) | 0x10);
            create_group::create_group()?
        } else {
            join_group::join_group()
        };
        if !joined {
            return Ok(false);
        }
        list_players::list_players()?;
        Ok(true)
    }
}
