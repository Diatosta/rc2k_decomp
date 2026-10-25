//! The network's game objects as Rust types, the same as IDA's structs of the same names
//! (`NetConnection`, `NetSession`) and
//! DirectPlay's that the game fills (`DPNAME`, `DPSESSIONDESC2`,
//! `DPCOMPOUNDADDRESSELEMENT`): their layouts, checked at compile time. The code reaches
//! their fields through `crate::field!`. The records in the network's own variables
//! (`NetPlayerBlock`, the session description, the name) are `net::store`'s.

use std::mem::{offset_of, size_of};

pub type Guid = [u8; 16];

/// `g_netConnectionTable`'s records (`net::connection_entry`).
#[repr(C)]
pub struct NetConnection {
    pub name: [u8; 64],
    pub service_provider: Guid,
    /// The connection's data (memory of the network's list).
    pub data: u32,
}

/// `g_netSessionTable`'s records (`net::session_entry`).
#[repr(C)]
pub struct NetSession {
    pub name: [u8; 64],
    pub instance: Guid,
    pub max_players: u32,
    pub players: u32,
    pub user: [u32; 4],
}

/// DirectPlay's `DPNAME`.
#[repr(C)]
pub struct DpName {
    pub size: u32,
    pub flags: u32,
    pub short_name: u32,
    pub long_name: u32,
}

/// DirectPlay's `DPSESSIONDESC2` (`g_netSessionDesc`).
#[repr(C)]
pub struct DpSessionDesc {
    pub size: u32,
    pub flags: u32,
    pub instance: Guid,
    pub application: Guid,
    pub max_players: u32,
    pub current_players: u32,
    pub name: u32,
    pub password: u32,
    pub reserved: [u32; 2],
    pub user: [u32; 4],
}

/// DirectPlay's `DPCOMPOUNDADDRESSELEMENT`.
#[repr(C)]
pub struct DpAddressElement {
    pub data_type: Guid,
    pub data_size: u32,
    pub data: u32,
}

const _: () = assert!(size_of::<NetConnection>() == 0x54);
const _: () = assert!(size_of::<NetSession>() == 0x68);
const _: () = assert!(offset_of!(NetSession, user) == 0x58);
const _: () = assert!(size_of::<DpName>() == 0x10);
const _: () = assert!(size_of::<DpSessionDesc>() == 0x50);
const _: () = assert!(offset_of!(DpSessionDesc, user) == 0x40);
const _: () = assert!(size_of::<DpAddressElement>() == 0x18);
