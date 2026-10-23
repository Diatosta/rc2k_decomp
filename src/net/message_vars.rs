//! The variables of the network's message plumbing (agent I, read 2026-10-19; a reading,
//! not seen in play): the receiving thread's loop (`net::take_messages`), DirectPlay's
//! `Receive` and `Send` wrappers, the system messages, the received messages' lock and the
//! wrappers `net::destroy_connection` calls. By address: the game's, or a self-test's (the
//! receiving thread and the game's thread read these while a test runs, so the tests'
//! copies have their references moved to test memory).

use crate::net::a;

/// Where the variables are.
#[derive(Clone, Copy, Debug)]
pub struct MessageVars {
    /// g_dplay 0x5AB288: the `IDirectPlay4A`, 0 without one.
    pub dplay: u32,
    /// g_dplayLobby 0x5AB28C: the `IDirectPlayLobby2A`.
    pub lobby: u32,
    /// g_netPlayerId 0x5AB270: this machine's DirectPlay player.
    pub player_id: u32,
    /// g_netSendTo 0x5AB274: the group the messages go to.
    pub send_to: u32,
    /// dword_5AB278: a received message's sender (0: DirectPlay's system message).
    pub from: u32,
    /// unk_5AB27C: its addressee.
    pub to: u32,
    /// dword_5AB280: its size (0 when none was taken).
    pub size: u32,
    /// unk_52A968: its bytes, 0x100.
    pub data: u32,
    /// dword_52ACFC: a session is open.
    pub session_open: u32,
    /// g_netMutex 0x52AB6C: the received messages' mutex.
    pub mutex: u32,
    /// dword_52A91C: the messages taken, counted.
    pub taken: u32,
    /// g_netReceived 0x52AD9C: each player's block, 0x804C bytes (+0x20 the DirectPlay id,
    /// from +0x34 the messages: `net::store_received`).
    pub received: u32,
    /// g_netPlayers 0x52AD98.
    pub players: u32,
    /// g_netSync 0x52A914 (a byte): the lobby's round, counted up at each lobby.
    pub sync: u32,
    /// g_netOwnRecord 0x52AD64 (its first byte: 0x10 the host, 1 ready).
    pub own: u32,
    /// word_52A916: the race-sync answer sent (0x41 or 0x51, the round).
    pub answer: u32,
    /// g_netAllRaced 0x52AD78.
    pub all_raced: u32,
    /// g_netRaceOver 0x52AD7C.
    pub race_over: u32,
    /// g_netSynced 0x52AD90: 8 bytes, 1 a player answered.
    pub synced: u32,
    /// dword_52AD8C: the sync table not cleared yet.
    pub sync_first: u32,
    /// dword_52AC2C: the stage item can be chosen.
    pub stage_item: u32,
}

/// (IDA address, length referenced, the field) for the tests' redirections.
const PLACES: [(usize, usize); 21] = [
    (0x5AB288, 4),
    (0x5AB28C, 4),
    (0x5AB270, 4),
    (0x5AB274, 4),
    (0x5AB278, 4),
    (0x5AB27C, 4),
    (0x5AB280, 4),
    (0x52A968, 1),
    (0x52ACFC, 4),
    (0x52AB6C, 4),
    (0x52A91C, 4),
    (0x52AD9C, 1),
    (0x52AD98, 4),
    (0x52A914, 1),
    (0x52AD64, 1),
    (0x52A916, 2),
    (0x52AD78, 4),
    (0x52AD7C, 4),
    (0x52AD90, 1),
    (0x52AD8C, 4),
    (0x52AC2C, 4),
];

impl MessageVars {
    fn from_list(v: [u32; 21]) -> Self {
        Self {
            dplay: v[0],
            lobby: v[1],
            player_id: v[2],
            send_to: v[3],
            from: v[4],
            to: v[5],
            size: v[6],
            data: v[7],
            session_open: v[8],
            mutex: v[9],
            taken: v[10],
            received: v[11],
            players: v[12],
            sync: v[13],
            own: v[14],
            answer: v[15],
            all_raced: v[16],
            race_over: v[17],
            synced: v[18],
            sync_first: v[19],
            stage_item: v[20],
        }
    }

    /// The game's.
    pub fn game() -> Self {
        Self::from_list(PLACES.map(|(x, _)| a(x)))
    }
}

pub fn get(at: u32) -> u32 {
    unsafe { (at as *const u32).read_volatile() }
}

pub fn set(at: u32, value: u32) {
    unsafe { (at as *mut u32).write_volatile(value) }
}

pub fn get8(at: u32) -> u8 {
    unsafe { (at as *const u8).read_volatile() }
}

pub fn set8(at: u32, value: u8) {
    unsafe { (at as *mut u8).write_volatile(value) }
}

// ---- the tests' side ----
