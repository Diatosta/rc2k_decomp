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
use crate::net::lobby::{co_create, copy_printable, log};
use crate::net::store::{NET, PlayerRecord};
use crate::net::{create_group, create_player, fill_own_player, join_group, list_players};
use crate::pointers::GamePtr;
use crate::race::viewer_screen_store::{VIEWER_PANEL, texts};

const BUFFER_TOO_SMALL: u32 = 0x8877_001E; // DPERR_BUFFERTOOSMALL

fn get(at: u32) -> Result<u32, BadInput> {
    GamePtr::new(at).read::<u32>("the lobby's connection settings")
}

/// The Rust version.
pub fn open_lobby() -> Result<bool, Refused> {
    log(VIEWER_PANEL.text_ptr(texts::NET_CREATE_DLOBBY), true, 2);
    if co_create(&NET.clsid_lobby, &NET.iid_lobby2a, &NET.lobby) != 0 {
        return Ok(false);
    }
    if NET.lobby_launched.get() != 1 {
        return Ok(true);
    }
    NET.settings_size.set(0);
    if crate::net::dp_objects::get_connection_settings(NET.lobby.get()) != BUFFER_TOO_SMALL {
        return Ok(false);
    }
    NET.settings.set(crate::net::lobby::take_lobby_settings(
        NET.settings_size.get(),
    )?);
    if crate::net::dp_objects::get_connection_settings(NET.lobby.get()) != 0 {
        return Ok(false);
    }
    GamePtr::new(get(NET.settings.get() + 8)? + 4)
        .write(0x44u32, "the lobby's connection settings")?;
    if crate::net::dp_objects::set_connection_settings(NET.lobby.get()) != 0 {
        return Ok(false);
    }
    if crate::net::dp_objects::connect(NET.lobby.get()) != 0 {
        return Ok(false);
    }
    if NET.dplay.get() == 0 || NET.dplay_2.get() == 0 {
        return Err(BadInput("a lobby's connection with no DirectPlay object").into());
    }
    let queried = crate::net::dp_objects::query_interface(NET.dplay.get(), &NET.iid_dplay4a.read());
    NET.dplay.set(*queried.as_ref().unwrap_or(&0));
    crate::net::dp_objects::release(NET.dplay_2.get());
    NET.dplay_2.set(0);
    if queried.is_err() {
        return Ok(false);
    }
    let name = get(get(NET.settings.get() + 0xC)? + 8)?;
    copy_printable(name, &NET.player_name, 0x1F)?;
    fill_own_player::fill_own_player(name)?;
    if !create_player::create_player()? {
        return Ok(false);
    }
    let joined = if get(NET.settings.get() + 4)? & 2 != 0 {
        NET.own
            .head
            .set(PlayerRecord::STATE, NET.own.state() | 0x10);
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
