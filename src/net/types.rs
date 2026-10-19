//! The network's game objects as Rust types, the same as IDA's structs of the same names
//! (`NetPlayerRecord`, `NetMessage`, `NetPlayerBlock`, `NetConnection`, `NetSession`) and
//! DirectPlay's that the game fills (`DPNAME`, `DPSESSIONDESC2`,
//! `DPCOMPOUNDADDRESSELEMENT`): their layouts, checked at compile time. The code reaches
//! their fields through `crate::field!`.

use std::mem::{offset_of, size_of};

pub type Guid = [u8; 16];

/// A player's 16 bytes of DirectPlay player data (`g_netOwnRecord` for this machine).
#[repr(C)]
pub struct NetPlayerRecord {
    /// 1 ready, 2 waiting, 4 racing, 0x10 the host.
    pub state: u8,
    pub car: u8,
    pub colours: u8,
    pub nationality: u8,
    pub stage: u8,
    pub direction: u8,
    pub race_number: u8,
    pub random: u8,
    pub race_time: u32,
    pub total: u32,
}

/// A received message (`net::store_received`): its length and bytes.
#[repr(C)]
pub struct NetMessage {
    pub field_0: u8,
    pub length: u8,
    pub bytes: [u8; 254],
}

/// `g_netReceived` (8 of them): a player's block.
#[repr(C)]
pub struct NetPlayerBlock {
    pub name: [u8; 32],
    /// Its DirectPlay id.
    pub id: u32,
    pub record: NetPlayerRecord,
    /// The messages received (`net::store_received`).
    pub count: u32,
    pub messages: [NetMessage; 64],
    pub field_4038: [u32; 2],
    /// Their copy for the race (`net::copy_received`, `net::apply_received`).
    pub copied_count: u32,
    pub copied: [NetMessage; 64],
    pub field_8044: [u32; 2],
}

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

const _: () = assert!(size_of::<NetPlayerRecord>() == 0x10);
const _: () = assert!(size_of::<NetMessage>() == 0x100);
const _: () = assert!(size_of::<NetPlayerBlock>() == 0x804C);
const _: () = assert!(offset_of!(NetPlayerBlock, id) == 0x20);
const _: () = assert!(offset_of!(NetPlayerBlock, record) == 0x24);
const _: () = assert!(offset_of!(NetPlayerBlock, count) == 0x34);
const _: () = assert!(offset_of!(NetPlayerBlock, messages) == 0x38);
const _: () = assert!(offset_of!(NetPlayerBlock, copied_count) == 0x4040);
const _: () = assert!(size_of::<NetConnection>() == 0x54);
const _: () = assert!(size_of::<NetSession>() == 0x68);
const _: () = assert!(offset_of!(NetSession, user) == 0x58);
const _: () = assert!(size_of::<DpName>() == 0x10);
const _: () = assert!(size_of::<DpSessionDesc>() == 0x50);
const _: () = assert!(offset_of!(DpSessionDesc, user) == 0x40);
const _: () = assert!(size_of::<DpAddressElement>() == 0x18);
