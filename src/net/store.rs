//! The network's variables, typed: `NetShared` (IDA 0x52A4B0 .. 0x5AB419): `nonetwork` and
//! `inetport`, the chat lines, the session's state (`session`, 1 while a network game runs,
//! which the race code reads too) and this machine's player, the send and receive buffers,
//! the connections and sessions listed, the received message stores (8 `PlayerBlock`s),
//! DirectPlay's objects, GUIDs and the chat zone's picture; `Button` (0x5AB490),
//! `net::draw_button`'s; and `NetMain`'s records (the item edited, this machine's settings,
//! the address being built, the session joined, the lobby's `LobbyVars`), which only the
//! game's thread reaches. The
//! installer's variables, the build notice and the lobby's registry names are
//! `install::store`'s.
//!
//! Structs of cells (what `Cells` needs), in two kinds, by thread (M3). The
//! receiving thread (T3, `net::receive_thread`) shares `NetShared`: atomic cells, plain
//! loads and stores (`Relaxed`), as the original's; the received blocks are written under
//! the game's mutex (`lock_received`). What only the game's thread (T0) reaches is plain
//! cells in `NetMain` (`LobbyVars` and the records below), and the DirectPlay and service-
//! provider GUIDs, never written, are constants.
//!
//! The receiving thread's reach, field by field (checked by grep, 2026-11-12):
//! - T3 writes, T0 reads: `receive_done`, `receiving`, `posted_result`, `posted_answered`,
//!   `messages_taken`, `sync_answer`, `all_raced`, `race_over`, `synced`, `sync_first`,
//!   `stage_item` (a host made), `own` (the host bit), `received` (a message stored, a chat
//!   sender's name), `chat`, `chat_line`, `chat_text` (`add_chat_line`), `receive_data`,
//!   `received_from` / `_to` / `_size`, `session` (a lost session: `destroy_connection`) and,
//!   with it, `dplay`, `lobby`, `session_open`, `memory`, `connections`, `sessions`,
//!   `connection_table`, `session_table`, `player_id`, `sync` (the round of a host's first
//!   call), and `players` (`close_session`, `release_*`, `free_messages`, `destroy_player`).
//! - T0 writes, T3 reads: `posting`, `posted_call`, `event`, `mutex`, `host_name`,
//!   `session_desc`, `send_to`, `players`, `dplay`.
//! - Read by the drawing threads: `session`, `player` (lights, hud icons).
//! - T0 only, plain (`LobbyVars`): `mode`, `sequence`, `first_host`, `send_buffer`,
//!   `show_gaps`, `data_size`, `racing`, `after_race`, `sync_clock`.
//! - Dead and removed: the `DPNAME`, `dplay_2`, `DPAID_TotalSize`, the chat zone's file name.

use crate::session::{Byte, Bytes, Dword, PlainByte, PlainBytes, PlainDword, Word, with_runs};
use rc2k_core::net::inbox::{Inbox, Spill};

const fn zeros<const N: usize>() -> [Dword; N] {
    [const { Dword::new(0) }; N]
}

/// the game's application GUID (`g_netAppGuid`), IDA 0x5AB300: never written.
pub const APP_GUID: [u8; 16] = with_runs::<16>(&[
    (0, b"\x80\xb0\xfa\xb5W\x09\xd1\x11\x91\xc0"),
    (11, b"\xc0\xf0\x1a\x9f\xb3"),
]);

/// `IID_IDirectPlay4A`, IDA 0x5AB310: never written.
pub const IID_DPLAY4A: [u8; 16] = with_runs::<16>(&[
    (0, b"A\xfe>\x13\xdc2\xd0\x11\x9c\xfb"),
    (11, b"\xa0\xc9\x0aC\xcb"),
]);

/// `IID_IDirectPlayLobby2A`, IDA 0x5AB320: never written.
pub const IID_LOBBY2A: [u8; 16] = with_runs::<16>(&[
    (0, b"\x80\xaf\xb4\x1b\x03\xa3\xd0\x11\x9cO"),
    (11, b"\xa0\xc9\x05B^"),
]);

/// `CLSID_DirectPlay`, IDA 0x5AB330: never written.
pub const CLSID_DPLAY: [u8; 16] = with_runs::<16>(&[
    (0, b" m\xeb\xd1#\x89\xd0\x11\x9d\x97"),
    (11, b"\xa0\xc9\x0aC\xcb"),
]);

/// `CLSID_DirectPlayLobby`, IDA 0x5AB340: never written.
pub const CLSID_LOBBY: [u8; 16] = with_runs::<16>(&[
    (0, b"\x10\xf8\xe8/\xa5\xb2\xd0\x11\xa7\x87"),
    (12, b"\xf8\x03\xab\xfc"),
]);

/// `DPSPGUID_IPX`, IDA 0x5AB350: never written.
pub const SP_IPX: [u8; 16] = with_runs::<16>(&[
    (1, b"\xc4[h,\x9d\xcf\x11\xa9\xcd"),
    (11, b"\xaa"),
    (13, b"h\x86\xe3"),
]);

/// `DPSPGUID_TCPIP`, IDA 0x5AB360: never written.
pub const SP_TCPIP: [u8; 16] = with_runs::<16>(&[
    (0, b"\xe0^\xe96w\x85\xcf\x11\x96\x0c"),
    (11, b"\x80\xc7SN\x82"),
]);

/// `DPSPGUID_SERIAL`, IDA 0x5AB370: never written.
pub const SP_SERIAL: [u8; 16] = with_runs::<16>(&[
    (0, b"`h\x1d\x0f\xd9\x88\xcf\x11\x9cN"),
    (11, b"\xa0\xc9\x05B^"),
]);

/// `DPSPGUID_MODEM`, IDA 0x5AB380: never written.
pub const SP_MODEM: [u8; 16] = with_runs::<16>(&[
    (0, b"`\xa7\xeaDh\xcb\xcf\x11\x9cN"),
    (11, b"\xa0\xc9\x05B^"),
]);

/// `DPAID_ServiceProvider`, IDA 0x5AB3A0: never written.
pub const AID_SERVICE_PROVIDER: [u8; 16] = with_runs::<16>(&[
    (0, b"\xc0\x16\xd9\x07\xaf\xe0\xcf\x11\x9cN"),
    (11, b"\xa0\xc9\x05B^"),
]);

/// `DPAID_INet`, IDA 0x5AB3E0: never written.
pub const AID_INET: [u8; 16] = with_runs::<16>(&[
    (0, b"\xa0M\xa5\xc4\xaf\xe0\xcf\x11\x9cN"),
    (11, b"\xa0\xc9\x05B^"),
]);

/// `DPAID_INetPort`, IDA 0x5AB3F0: never written.
pub const AID_INET_PORT: [u8; 16] = with_runs::<16>(&[
    (0, b"AER\xe4\xa5\x8e\xd1\x11\x8a\x96"),
    (11, b"`\x97\xb0\x14\x11"),
]);

/// 0x5AB25C: `aSusanivanovafa`: "susanivanovafanclub", the group's name; never written.
pub const GROUP_NAME: [u8; 20] = with_runs::<20>(&[(0, b"susanivanovafanclub")]);

/// A player's 16 bytes of DirectPlay player data (`NetPlayerRecord`; `g_netOwnRecord` for
/// this machine).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct PlayerRecord {
    /// +0 the state bits (1 ready, 2 waiting, 4 racing, 0x10 the host), +1 the car, +2 its
    /// colours, +3 the nationality, +4 the stage, +5 its direction, +6 the race number, +7 a
    /// random number.
    pub head: Bytes<8>,
    /// +8: the race's time.
    pub race_time: Dword,
    /// +0xC: the total.
    pub total: Dword,
}

impl PlayerRecord {
    pub const STATE: usize = 0;
    pub const CAR: usize = 1;
    pub const COLOURS: usize = 2;
    pub const STAGE: usize = 4;
    pub const DIRECTION: usize = 5;

    pub const fn zero() -> Self {
        Self {
            head: Bytes::new([0; 8]),
            race_time: Dword::new(0),
            total: Dword::new(0),
        }
    }

    /// The state bits.
    pub fn state(&self) -> u8 {
        self.head.get(Self::STATE)
    }

    /// The record's 16 bytes, as DirectPlay's player data.
    pub fn bytes(&self) -> [u8; 16] {
        let mut b = [0u8; 16];
        b[..8].copy_from_slice(&self.head.read());
        b[8..12].copy_from_slice(&self.race_time.get().to_le_bytes());
        b[12..].copy_from_slice(&self.total.get().to_le_bytes());
        b
    }

    /// The record's first `data.len()` bytes (at most 16) set from `data`.
    pub fn set_bytes(&self, data: &[u8]) {
        let mut b = self.bytes();
        let n = data.len().min(16);
        b[..n].copy_from_slice(&data[..n]);
        for (k, &v) in b[..8].iter().enumerate() {
            self.head.set(k, v);
        }
        self.race_time
            .set(u32::from_le_bytes([b[8], b[9], b[10], b[11]]));
        self.total
            .set(u32::from_le_bytes([b[12], b[13], b[14], b[15]]));
    }
}

/// A player's block of `g_netReceived` (`NetPlayerBlock`).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct PlayerBlock {
    /// The player's name.
    pub name: Bytes<32>,
    /// Its DirectPlay id.
    pub id: Dword,
    /// Its record.
    pub record: PlayerRecord,
    /// The messages received (`net::store_received`): the records held.
    pub count: Dword,
    /// The 0x40 records of 0x100 bytes (+0 the sequence byte doubled, +1 the length, +2 the
    /// bytes).
    pub messages: Bytes<0x4000>,
    /// The oldest sequence still wanted.
    pub oldest: Byte,
    /// The newest sequence seen.
    pub newest: Byte,
    /// Two bytes that align `last_time` (`Cells` takes no padding); nothing
    /// reads or writes them.
    pub _pad_cells: Bytes<2>,
    /// The time of the last message.
    pub last_time: Dword,
    /// The count of their copy for the race (`net::copy_received`).
    pub copied_count: Dword,
    /// Their copy (`net::apply_received`).
    pub copied: Bytes<0x4000>,
}

impl PlayerBlock {
    pub const fn zero() -> Self {
        Self {
            name: Bytes::new([0; 32]),
            id: Dword::new(0),
            record: PlayerRecord::zero(),
            count: Dword::new(0),
            messages: Bytes::new([0; 0x4000]),
            oldest: Byte::new(0),
            newest: Byte::new(0),
            _pad_cells: Bytes::new([0; 2]),
            last_time: Dword::new(0),
            copied_count: Dword::new(0),
            copied: Bytes::new([0; 0x4000]),
        }
    }

    /// The inbox `rc2k_core::net::inbox` works on, read field by field.
    pub fn inbox(&self) -> Inbox {
        Inbox {
            count: self.count.get(),
            records: self.messages.read().to_vec(),
            oldest: self.oldest.get(),
            newest: self.newest.get(),
            time: self.last_time.get(),
        }
    }

    /// `inbox` written back, field by field.
    pub fn set_inbox(&self, inbox: &Inbox) -> Result<(), crate::errors::BadInput> {
        self.count.set(inbox.count);
        self.messages
            .write_at(0, &inbox.records, "a player's records")?;
        self.oldest.set(inbox.oldest);
        self.newest.set(inbox.newest);
        self.last_time.set(inbox.time);
        Ok(())
    }

    /// What a message put into a full inbox writes past the last record: the sequence byte
    /// over `oldest`, the length over `newest`, and the bytes on over two that are not kept,
    /// then `last_time`, `copied_count` and the first of `copied`.
    pub fn write_spill(&self, spill: &Spill) {
        self.oldest.set(spill.sequence);
        self.newest.set(spill.length);
        let (mut time, mut copied_count) = (
            self.last_time.get().to_le_bytes(),
            self.copied_count.get().to_le_bytes(),
        );
        for (k, &byte) in spill.bytes.iter().enumerate() {
            match k {
                0..=1 => {}
                2..=5 => time[k - 2] = byte,
                6..=9 => copied_count[k - 6] = byte,
                _ => self.copied.set(k - 10, byte),
            }
        }
        self.last_time.set(u32::from_le_bytes(time));
        self.copied_count.set(u32::from_le_bytes(copied_count));
    }
}

/// `bytes` into `cells`' dwords, little-endian (a GUID's 16 bytes into four).
pub fn put_bytes(cells: &[Dword], bytes: &[u8]) {
    for (cell, b) in cells.iter().zip(bytes.chunks(4)) {
        let mut v = cell.get().to_le_bytes();
        v[..b.len()].copy_from_slice(b);
        cell.set(u32::from_le_bytes(v));
    }
}

/// DirectPlay's `DPSESSIONDESC2` (80 bytes).
#[derive(rc2k_derive::Cells)]
pub struct SessionDesc {
    pub size: Dword,
    pub flags: Dword,
    /// The session's GUID.
    pub instance: [Dword; 4],
    /// The application's GUID.
    pub application: [Dword; 4],
    pub max_players: Dword,
    pub current_players: Dword,
    pub name: Dword,
    pub password: Dword,
    pub reserved: [Dword; 2],
    pub user: [Dword; 4],
}

impl SessionDesc {
    pub const fn zero() -> Self {
        Self {
            size: Dword::new(0),
            flags: Dword::new(0),
            instance: zeros(),
            application: zeros(),
            max_players: Dword::new(0),
            current_players: Dword::new(0),
            name: Dword::new(0),
            password: Dword::new(0),
            reserved: zeros(),
            user: zeros(),
        }
    }

    /// Every field 0 (`system::zero_memory` of its 0x50 bytes).
    pub fn clear(&self) {
        for d in [
            &self.size,
            &self.flags,
            &self.max_players,
            &self.current_players,
            &self.name,
            &self.password,
        ] {
            d.set(0);
        }
        for d in self
            .instance
            .iter()
            .chain(&self.application)
            .chain(&self.reserved)
            .chain(&self.user)
        {
            d.set(0);
        }
    }
}

/// IDA 0x52a4b4 ..: the edit variables, the game thread's.
#[derive(rc2k_derive::Cells)]
pub struct EditVars {
    /// 0x52A4B4: `g_netEditText`: the text of the item edited, by number
    /// (`net::begin_edit::EditedText`; the original keeps its address).
    pub edit_text: PlainDword,
    /// 0x52A4B8: `g_netEdited`: the item edited (-1: none).
    pub edited: PlainDword,
    /// 0x52A4BC: `dword_52A4BC`: the widest the edited text may be.
    pub edit_width: PlainDword,
    /// 0x52A4C0: `dword_52A4C0`: the font the edited text is measured with (the original
    /// keeps its address).
    pub edit_font: crate::gfx::typefaces::FontCell,
    /// 0x52A4C4: `dword_52A4C4`: the time of the players' last listing.
    pub clock_a: PlainDword,
    /// 0x52A4C8: `g_netChatInput`: what this player types into the chat.
    pub chat_input: PlainBytes<64>,
}

impl EditVars {
    pub const fn new() -> Self {
        Self {
            edit_text: PlainDword::new(0),
            edited: PlainDword::new(0),
            edit_width: PlainDword::new(0),
            // 0 in the image: the first font's number; read only once an edit has set it.
            edit_font: crate::gfx::typefaces::FontCell::new(
                crate::gfx::typefaces::FontId::numbered(0),
            ),
            clock_a: PlainDword::new(0),
            chat_input: PlainBytes::new([0; 64]),
        }
    }
}

/// IDA 0x52ab70 ..: the setup variables, the game thread's.
#[derive(rc2k_derive::Cells)]
pub struct SetupVars {
    /// 0x52AB70: `byte_52AB70`: this player's name.
    pub player_name: PlainBytes<32>,
    /// 0x52AB90: `g_machineName`: the machine's name (kept in `var\ral.dat`).
    pub machine_name: PlainBytes<64>,
    /// 0x52ABD0: `g_netMostPlayers`: the most players a hosted session takes (kept).
    pub most_players: PlainDword,
    /// 0x52ABD4: `unk_52ABD4`: the session's password.
    pub session_password: PlainBytes<16>,
    /// 0x52ABE4: `unk_52ABE4`: the host's address (TCP/IP), kept.
    pub host_address: PlainBytes<64>,
    /// 0x52AC24: `dword_52AC24`: the TCP/IP port (the `inetport` setting, kept).
    pub port: PlainDword,
    /// 0x52AC28: `dword_52AC28`: the lobby's car item can be chosen.
    pub car_item: PlainDword,
}

impl SetupVars {
    pub const fn new() -> Self {
        Self {
            player_name: PlainBytes::new([0; 32]),
            machine_name: PlainBytes::new([0; 64]),
            most_players: PlainDword::new(0x8),
            session_password: PlainBytes::new(with_runs::<16>(&[(0, b"\x17\x00\x00\x00\x01")])),
            host_address: PlainBytes::new([0; 64]),
            port: PlainDword::new(0),
            car_item: PlainDword::new(0x1),
        }
    }
}

/// IDA 0x52ac3c ..: the address variables, the game thread's.
#[derive(rc2k_derive::Cells)]
pub struct AddressVars {
    /// 0x52AC3C: `dword_52AC3C`: the address elements added.
    pub elements: PlainDword,
    /// 0x52AC40: `dword_52AC40`: the compound address (memory of the network's list; 0 now:
    /// `net::tables::NET_TABLES`).
    pub address: PlainDword,
    /// 0x52AC44: `bytes`: the compound address's size.
    pub address_size: PlainDword,
    /// 0x52AC48: `unk_52AC48`: the address elements, 0x18 bytes each (a DPCOMPOUNDADDRESSELEMENT: the data type's GUID, the size, the data).
    pub element_table: PlainBytes<76>,
    /// 0x52AC94: `unk_52AC94`: the saved connection's GUID (kept).
    pub saved_connection: PlainBytes<16>,
    /// 0x52ACA4: `unk_52ACA4`: the saved connection's name.
    pub saved_name: PlainBytes<64>,
}

impl AddressVars {
    pub const fn new() -> Self {
        Self {
            elements: PlainDword::new(0),
            address: PlainDword::new(0),
            address_size: PlainDword::new(0),
            element_table: PlainBytes::new([0; 76]),
            saved_connection: PlainBytes::new([0; 16]),
            saved_name: PlainBytes::new([0; 64]),
        }
    }
}

/// IDA 0x52ad00 ..: the joined variables, the game thread's.
#[derive(rc2k_derive::Cells)]
pub struct JoinedVars {
    /// 0x52AD00: `unk_52AD00`: the joined session's GUID.
    pub joined: PlainBytes<16>,
    /// 0x52AD10: `unk_52AD10`: its name.
    pub joined_name: PlainBytes<64>,
    /// 0x52AD50: `dword_52AD50`: its most players.
    pub joined_most: PlainDword,
    /// 0x52AD54: `unk_52AD54`: its password.
    pub joined_password: PlainBytes<16>,
}

impl JoinedVars {
    pub const fn new() -> Self {
        Self {
            joined: PlainBytes::new([0; 16]),
            joined_name: PlainBytes::new([0; 64]),
            joined_most: PlainDword::new(0),
            joined_password: PlainBytes::new([0; 16]),
        }
    }
}

/// The lobby's variables, the game thread's (IDA 0x52A908 .. 0x52AD88).
#[derive(rc2k_derive::Cells)]
pub struct LobbyVars {
    /// 0x52A908: `g_netMode`: 0 the settings, 1 host, 2 join, 3 lobby.
    pub mode: PlainDword,
    /// 0x52A924: `dword_52A924`: the first player not the host.
    pub first_host: PlainDword,
    /// 0x52AA68: `g_netSendBuffer`: the message being sent.
    pub send_buffer: PlainBytes<256>,
    /// 0x52AC30: `dword_52AC30`: times shown as gaps.
    pub show_gaps: PlainDword,
    /// 0x52AD74: `dword_52AD74`: the size of the player data listed.
    pub data_size: PlainDword,
    /// 0x52AD80: `g_netAfterRace`: back in the lobby after a race.
    pub after_race: PlainDword,
    /// 0x52AD84: `g_netRacing`: the players are not listed again.
    pub racing: PlainDword,
    /// 0x52AD88: `dword_52AD88`: the time of the last sync message.
    pub sync_clock: PlainDword,
    /// 0x52A915: `g_netSequence`: the sequence byte of the messages sent.
    pub sequence: PlainByte,
    /// Three bytes that fill the record's last dword (`Cells` takes no padding); nothing
    /// reads or writes them.
    pub _pad_cells: PlainBytes<3>,
}

impl LobbyVars {
    pub const fn new() -> Self {
        Self {
            mode: PlainDword::new(0x3),
            first_host: PlainDword::new(0),
            send_buffer: PlainBytes::new([0; 256]),
            show_gaps: PlainDword::new(0),
            data_size: PlainDword::new(0),
            after_race: PlainDword::new(0),
            racing: PlainDword::new(0),
            sync_clock: PlainDword::new(0),
            sequence: PlainByte::new(128),
            _pad_cells: PlainBytes::new([0; 3]),
        }
    }
}

/// The network's variables only the game's thread reaches, in plain cells: what is edited,
/// this machine's settings, the address being built, the session joined and the lobby's.
/// The receiving thread reads and writes none of them (it works on `NetShared`).
pub struct NetMain {
    pub edit: EditVars,
    pub setup: SetupVars,
    pub address: AddressVars,
    pub joined: JoinedVars,
    pub lobby: LobbyVars,
}

impl NetMain {
    pub const fn new() -> Self {
        NetMain {
            edit: EditVars::new(),
            setup: SetupVars::new(),
            address: AddressVars::new(),
            joined: JoinedVars::new(),
            lobby: LobbyVars::new(),
        }
    }
}

/// IDA 0x52A4B0 .. 0x5AB419.
#[derive(rc2k_derive::Cells)]
pub struct NetShared {
    /// 0x52A4B0: `g_netReceiveDone`: set when the receiving thread has ended; also the `nonetwork` setting's variable (`settings::read_settings`).
    pub receive_done: Dword,
    /// 0x52A508: `g_netChat`: the chat's lines (0x300 bytes).
    pub chat: Bytes<768>,
    /// 0x52A808: `byte_52A808`: a chat line being cut.
    pub chat_line: Bytes<128>,
    /// 0x52A888: `word_52A888`: a chat message "name> text".
    pub chat_text: Bytes<128>,
    /// 0x52A910: `g_netSession`: 1 while a network game runs; the race code reads it as the split screen's switch (one car's view chosen).
    pub session: Dword,
    /// 0x52A914: `g_netSync`: the sync round, counted up at each lobby.
    pub sync: Byte,
    /// One byte that aligns `sync_answer` (`Cells` takes no padding); nothing reads or writes it.
    pub _pad_cells: Bytes<1>,
    /// 0x52A916: `g_netSyncAnswer`: the race-sync answer sent (0x41 or 0x51, the round).
    pub sync_answer: Word,
    /// 0x52A918: `g_netReceiving`: the receiving thread is busy.
    pub receiving: Dword,
    /// 0x52A91C: `g_netMessagesTaken`: the messages the receiving thread took, counted.
    pub messages_taken: Dword,
    /// 0x52A920: `g_netPlayer`: this machine's player (its car's number); the race code reads it as the car chosen.
    pub player: Dword,
    /// 0x52A928: `g_netMemory`: the network's memory list (`memory::List`, 16 bytes).
    pub memory: [Dword; 4],
    /// 0x52A968: `unk_52A968`: the bytes of the message `net::receive_message` took.
    pub receive_data: Bytes<256>,
    /// 0x52AB68: `g_netEvent`: the receiving thread's event (a handle).
    pub event: Dword,
    /// 0x52AB6C: `g_netMutex`: the mutex of the received messages (a handle).
    pub mutex: Dword,
    /// 0x52AC2C: `g_netStageItem`: the lobby's stage item can be chosen.
    pub stage_item: Dword,
    /// 0x52AC34: `g_netConnections`: the connections listed.
    pub connections: Dword,
    /// 0x52AC38: `g_netConnectionTable`: the connection records (0x54 bytes each; an address,
    /// 0 now: `net::tables::NET_TABLES`).
    pub connection_table: Dword,
    /// Not the original's: the machine's name, copied here by `net::host_session` before the
    /// session is opened on the receiving thread (`posted_open_host`), which may not reach
    /// the game thread's settings.
    pub host_name: Bytes<64>,
    /// 0x52ACE4: `g_netPosting`: DirectPlay's calls are made on the receiving thread.
    pub posting: Dword,
    /// 0x52ACE8: `g_netPostedCall`: the call posted to the receiving thread.
    pub posted_call: Dword,
    /// 0x52ACEC: `g_netPostedResult`: its result.
    pub posted_result: Dword,
    /// 0x52ACF0: `g_netPostedAnswered`: the receiving thread has answered.
    pub posted_answered: Dword,
    /// 0x52ACF4: `g_netSessions`: the sessions listed.
    pub sessions: Dword,
    /// 0x52ACF8: `g_netSessionTable`: the session records (0x68 bytes each; an address, 0
    /// now: `net::tables::NET_TABLES`).
    pub session_table: Dword,
    /// 0x52ACFC: `g_netSessionOpen`.
    pub session_open: Dword,
    /// 0x52AD64: `g_netOwnRecord`: this machine's player record.
    pub own: PlayerRecord,
    /// 0x52AD78: `g_netAllRaced`: every player is ready.
    pub all_raced: Dword,
    /// 0x52AD7C: `g_netRaceOver`: every player has finished.
    pub race_over: Dword,
    /// 0x52AD8C: `g_netSyncFirst`: the sync table is not cleared yet.
    pub sync_first: Dword,
    /// 0x52AD90: `g_netSynced`: a byte a player, 1 when the player answered the sync.
    pub synced: Bytes<8>,
    /// 0x52AD98: `g_netPlayers`: the players.
    pub players: Dword,
    /// 0x52AD9C: `g_netReceived`: the players' blocks.
    pub received: [PlayerBlock; 8],
    /// 0x5AB270: `g_netPlayerId`: this player's DirectPlay id.
    pub player_id: Dword,
    /// 0x5AB274: `g_netSendTo`: the group the messages go to.
    pub send_to: Dword,
    /// 0x5AB278: `g_netReceivedFrom`: the sender of the message taken.
    pub received_from: Dword,
    /// 0x5AB27C: `unk_5AB27C`: its addressee.
    pub received_to: Dword,
    /// 0x5AB280: `g_netReceivedSize`: the size of the buffer / the message taken.
    pub received_size: Dword,
    /// 0x5AB288: `g_dplay`: the `IDirectPlay4A` object.
    pub dplay: Dword,
    /// 0x5AB28C: `g_dplayLobby`: the `IDirectPlayLobby2A` object.
    pub lobby: Dword,
    /// 0x5AB2AC: `g_netSessionDesc`: a `DPSESSIONDESC2`.
    pub session_desc: SessionDesc,
}

impl NetShared {
    pub const fn new() -> Self {
        NetShared {
            receive_done: Dword::new(0),
            chat: Bytes::new([0; 768]),
            chat_line: Bytes::new([0; 128]),
            chat_text: Bytes::new([0; 128]),
            session: Dword::new(0),
            sync: Byte::new(0),
            _pad_cells: Bytes::new([0; 1]),
            sync_answer: Word::new(0),
            receiving: Dword::new(0),
            messages_taken: Dword::new(0),
            player: Dword::new(0),
            memory: zeros(),
            receive_data: Bytes::new([0; 256]),
            event: Dword::new(0),
            mutex: Dword::new(0),
            stage_item: Dword::new(0x1),
            connections: Dword::new(0),
            connection_table: Dword::new(0),
            host_name: Bytes::new([0; 64]),
            posting: Dword::new(0),
            posted_call: Dword::new(0),
            posted_result: Dword::new(0),
            posted_answered: Dword::new(0),
            sessions: Dword::new(0),
            session_table: Dword::new(0),
            session_open: Dword::new(0),
            own: PlayerRecord::zero(),
            all_raced: Dword::new(0),
            race_over: Dword::new(0),
            sync_first: Dword::new(0x1),
            synced: Bytes::new([0; 8]),
            players: Dword::new(0),
            received: [const { PlayerBlock::zero() }; 8],
            player_id: Dword::new(0),
            send_to: Dword::new(0),
            received_from: Dword::new(0),
            received_to: Dword::new(0),
            received_size: Dword::new(0),
            dplay: Dword::new(0),
            lobby: Dword::new(0),
            session_desc: SessionDesc::zero(),
        }
    }
}

impl NetShared {
    /// Player `n`'s block of `received`. `players` never passes 8 (`net::player_found` lists
    /// no more, and is the only one to count it up); a larger number is refused.
    pub fn player_block(&self, n: u32) -> Result<&PlayerBlock, crate::errors::BadInput> {
        self.received
            .get(n as usize)
            .ok_or(crate::errors::BadInput("a player number of 8 or more"))
    }

    /// Player `n`'s block for a core's closure, which cannot carry a refusal: a number the
    /// block table has no room for ends the program with it.
    pub fn player_block_or_end(&self, n: u32) -> &PlayerBlock {
        self.player_block(n)
            .unwrap_or_else(|e| crate::fatal::raise::end("a player's block", e))
    }

    /// Player `n`'s byte of `synced` (the players are at most 8).
    pub fn synced(&self, n: usize) -> Result<u8, crate::errors::BadInput> {
        Ok(self.synced.read_at(n, 1, "a player's synced byte")?[0])
    }

    pub fn set_synced(&self, n: usize, value: u8) -> Result<(), crate::errors::BadInput> {
        self.synced.write_at(n, &[value], "a player's synced byte")
    }
}

/// IDA 0x5AB490 .. 0x5AB4AC.
#[derive(rc2k_derive::Cells)]
pub struct Button {
    /// 0x5AB490: `dword_5AB490`: `net::draw_button`'s string.
    pub text: PlainDword,
    /// 0x5AB494: `dword_5AB494`: its flags.
    pub flags: PlainDword,
    /// 0x5AB498: `x`: its x.
    pub x: PlainDword,
    /// 0x5AB49C: `y`: its y.
    pub y: PlainDword,
    /// 0x5AB4A0: `dword_5AB4A0`: its width.
    pub width: PlainDword,
    /// 0x5AB4A4: `dword_5AB4A4`: its frame's tiles.
    pub tiles: PlainDword,
    /// 0x5AB4A8: `dword_5AB4A8`: its item.
    pub item: PlainDword,
}

impl Button {
    pub const fn new() -> Self {
        Button {
            text: PlainDword::new(0),
            flags: PlainDword::new(0),
            x: PlainDword::new(0),
            y: PlainDword::new(0),
            width: PlainDword::new(0),
            tiles: PlainDword::new(0),
            item: PlainDword::new(0),
        }
    }
}
