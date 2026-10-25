//! DirectPlay, called from Rust (the platform): `CoCreateInstance` and the
//! methods of the game's objects (`g_dplay` 0x5AB288, an `IDirectPlay4A`; `g_dplayLobby`
//! 0x5AB28C, an `IDirectPlayLobby2A`; `dword_5AB284`). Each call takes the object from its
//! variable, as the original does at the call, and gives the method's result. ecx and edx,
//! which a method may change, are left as they were by the Rust versions: no original reads
//! them after one of these calls.
//!
//! The methods by offset (IDirectPlay4A): 0 QueryInterface, 8 Release, 0xC AddPlayerToGroup,
//! 0x10 Close, 0x14 CreateGroup, 0x18 CreatePlayer, 0x1C DeletePlayerFromGroup, 0x24 DestroyPlayer, 0x2C EnumGroups,
//! 0x30 EnumPlayers, 0x34 EnumSessions, 0x50 GetPlayerData, 0x60 Open, 0x64 Receive, 0x68 Send,
//! 0x74 SetPlayerData, 0x7C SetSessionDesc, 0x8C EnumConnections, 0x98 InitializeConnection;
//! (IDirectPlayLobby2A) 8 Release, 0xC Connect, 0x20 GetConnectionSettings,
//! 0x30 SetConnectionSettings, 0x38 CreateCompoundAddress.

pub const QUERY_INTERFACE: u32 = 0x00;
pub const RELEASE: u32 = 0x08;
pub const ADD_PLAYER_TO_GROUP: u32 = 0x0C;
pub const CONNECT: u32 = 0x0C;
pub const CLOSE: u32 = 0x10;
pub const DELETE_PLAYER_FROM_GROUP: u32 = 0x1C;
pub const DESTROY_PLAYER: u32 = 0x24;
pub const CREATE_GROUP: u32 = 0x14;
pub const CREATE_PLAYER: u32 = 0x18;
pub const GET_CONNECTION_SETTINGS: u32 = 0x20;
pub const ENUM_GROUPS: u32 = 0x2C;
pub const ENUM_PLAYERS: u32 = 0x30;
pub const SET_CONNECTION_SETTINGS: u32 = 0x30;
pub const ENUM_SESSIONS: u32 = 0x34;
pub const CREATE_COMPOUND_ADDRESS: u32 = 0x38;
pub const GET_PLAYER_DATA: u32 = 0x50;
pub const OPEN: u32 = 0x60;
pub const RECEIVE: u32 = 0x64;
pub const SEND: u32 = 0x68;
pub const SET_PLAYER_DATA: u32 = 0x74;
pub const SET_SESSION_DESC: u32 = 0x7C;
pub const ENUM_CONNECTIONS: u32 = 0x8C;
pub const INITIALIZE_CONNECTION: u32 = 0x98;

/// The method at `offset` of the object at `object` (an address), with `args` after the
/// object: its result (an HRESULT).
pub unsafe fn method(object: u32, offset: u32, args: &[u32]) -> u32 {
    type F0 = unsafe extern "system" fn(u32) -> u32;
    type F1 = unsafe extern "system" fn(u32, u32) -> u32;
    type F2 = unsafe extern "system" fn(u32, u32, u32) -> u32;
    type F3 = unsafe extern "system" fn(u32, u32, u32, u32) -> u32;
    type F4 = unsafe extern "system" fn(u32, u32, u32, u32, u32) -> u32;
    type F5 = unsafe extern "system" fn(u32, u32, u32, u32, u32, u32) -> u32;
    type F6 = unsafe extern "system" fn(u32, u32, u32, u32, u32, u32, u32) -> u32;
    unsafe {
        let f = crate::audio::com::method(object, offset);
        let o = object;
        match *args {
            [] => std::mem::transmute::<usize, F0>(f)(o),
            [a] => std::mem::transmute::<usize, F1>(f)(o, a),
            [a, b] => std::mem::transmute::<usize, F2>(f)(o, a, b),
            [a, b, c] => std::mem::transmute::<usize, F3>(f)(o, a, b, c),
            [a, b, c, d] => std::mem::transmute::<usize, F4>(f)(o, a, b, c, d),
            [a, b, c, d, e] => std::mem::transmute::<usize, F5>(f)(o, a, b, c, d, e),
            [a, b, c, d, e, h] => std::mem::transmute::<usize, F6>(f)(o, a, b, c, d, e, h),
            _ => unreachable!("no DirectPlay method takes more than six arguments"),
        }
    }
}

/// A method of the object in the variable `var`.
pub unsafe fn call(var: &crate::session::Dword, offset: u32, args: &[u32]) -> u32 {
    unsafe { method(var.get(), offset, args) }
}

/// `CoCreateInstance(class, 0, CLSCTX_INPROC_SERVER, interface, out)`; in the standalone
/// program DirectPlay's objects made in Rust (`net::dp_objects`).
pub unsafe fn create_instance(class: u32, interface: u32, out: u32) -> i32 {
    unsafe { crate::net::dp_objects::create(class, interface, out) }
}
