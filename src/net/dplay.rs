//! DirectPlay's `CoCreateInstance`, called from Rust (the platform). The game's objects
//! (`g_dplay` 0x5AB288, an `IDirectPlay4A`; `g_dplayLobby` 0x5AB28C, an
//! `IDirectPlayLobby2A`; `dword_5AB284`) are `net::dp_objects`' ids, their methods its
//! functions (phase 11, 2026-10-26: before, called through the objects' tables by offset:
//! (IDirectPlay4A) 0 QueryInterface, 8 Release, 0xC AddPlayerToGroup, 0x10 Close, 0x14
//! CreateGroup, 0x18 CreatePlayer, 0x1C DeletePlayerFromGroup, 0x24 DestroyPlayer, 0x2C
//! EnumGroups, 0x30 EnumPlayers, 0x34 EnumSessions, 0x50 GetPlayerData, 0x60 Open, 0x64
//! Receive, 0x68 Send, 0x74 SetPlayerData, 0x7C SetSessionDesc, 0x8C EnumConnections, 0x98
//! InitializeConnection; (IDirectPlayLobby2A) 8 Release, 0xC Connect, 0x20
//! GetConnectionSettings, 0x30 SetConnectionSettings, 0x38 CreateCompoundAddress).

/// `CoCreateInstance(class, 0, CLSCTX_INPROC_SERVER, interface, out)`; in the standalone
/// program DirectPlay's objects made in Rust (`net::dp_objects`): the object, or the error.
pub fn create_instance(class: &[u8; 16], interface: &[u8; 16]) -> Result<u32, u32> {
    crate::net::dp_objects::create(class, interface)
}
