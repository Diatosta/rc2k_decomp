//! The network's variables, typed: `NetShared` (chat,
//! send and receive buffers, the received message stores and the chat zone's picture), `Button` (`net::draw_button`'s) and
//! `NetMain`'s records (the item edited, this machine's settings, the address being built, the
//! session joined, the lobby's `LobbyVars`), which only the game's thread reaches.
//!
//! Two kinds of cell structs, by thread. The receiving thread shares `NetShared`: atomic cells
//! with plain relaxed loads and stores, the receiving thread's own in `ReceiverState`; the
//! players' inboxes are behind the received messages' lock (`Inboxes`). What only the game's
//! thread reaches is plain cells in `NetMain`; the DirectPlay and service provider GUIDs are
//! never written and are constants.
//!
//! The receiving thread's reach, field by field:
//! - It writes, the game thread reads: `receive_done`, `receiver` (`receiving`, `posted_result`,
//!   `posted_answered`, `messages_taken`, `sync_answer`, `sync`: the round of a host's first
//!   call), `all_raced`, `race_over`, `synced`, `sync_first`, `stage_item` (a host made), `own`
//!   (the host bit), `received`, `chat`, `chat_line`, `chat_text`, and with a lost session
//!   (`destroy_connection`) the `Link`'s objects and lists.
//! - The game thread writes, it reads: `receiver.posting`, `receiver.posted_call`, `event`; and
//!   the `Link`'s `host_name`, `session_desc`, `send_to`, `dplay`.
//! - The network game's on / off, this machine's player and the players' count are
//!   `Net::race_slot` (`rc2k_core::net::net_race`), read by the drawing threads too (lights, HUD
//!   icons).
//! - Game thread only (`LobbyVars`): `mode`, `sequence`, `first_host`, `send_buffer`,
//!   `show_gaps`, `data_size`, `racing`, `after_race`, `sync_clock`.

use crate::memory::session::{
    Byte, Bytes, Dword, Flag, PlainByte, PlainBytes, PlainDword, Word, with_runs,
};
use rc2k_core::net::inbox::Inbox;
use rc2k_core::net::players::State;
use rc2k_core::net::record::{self, Record};

pub(crate) const fn zeros<const N: usize>() -> [Dword; N] {
    [const { Dword::new(0) }; N]
}

/// The game's application GUID: never written.
pub const APP_GUID: [u8; 16] = with_runs::<16>(&[
    (0, b"\x80\xb0\xfa\xb5W\x09\xd1\x11\x91\xc0"),
    (11, b"\xc0\xf0\x1a\x9f\xb3"),
]);

/// `IID_IDirectPlay4A`: never written.
pub(super) const IID_DPLAY4A: [u8; 16] = with_runs::<16>(&[
    (0, b"A\xfe>\x13\xdc2\xd0\x11\x9c\xfb"),
    (11, b"\xa0\xc9\x0aC\xcb"),
]);

/// `IID_IDirectPlayLobby2A`: never written.
pub(super) const IID_LOBBY2A: [u8; 16] = with_runs::<16>(&[
    (0, b"\x80\xaf\xb4\x1b\x03\xa3\xd0\x11\x9cO"),
    (11, b"\xa0\xc9\x05B^"),
]);

/// `CLSID_DirectPlay`: never written.
pub(super) const CLSID_DPLAY: [u8; 16] = with_runs::<16>(&[
    (0, b" m\xeb\xd1#\x89\xd0\x11\x9d\x97"),
    (11, b"\xa0\xc9\x0aC\xcb"),
]);

/// `CLSID_DirectPlayLobby`: never written.
pub(super) const CLSID_LOBBY: [u8; 16] = with_runs::<16>(&[
    (0, b"\x10\xf8\xe8/\xa5\xb2\xd0\x11\xa7\x87"),
    (12, b"\xf8\x03\xab\xfc"),
]);

/// `DPSPGUID_TCPIP`: never written.
pub const SP_TCPIP: [u8; 16] = with_runs::<16>(&[
    (0, b"\xe0^\xe96w\x85\xcf\x11\x96\x0c"),
    (11, b"\x80\xc7SN\x82"),
]);

/// `DPSPGUID_SERIAL`: never written.
pub(super) const SP_SERIAL: [u8; 16] = with_runs::<16>(&[
    (0, b"`h\x1d\x0f\xd9\x88\xcf\x11\x9cN"),
    (11, b"\xa0\xc9\x05B^"),
]);

/// `DPSPGUID_MODEM`: never written.
pub(super) const SP_MODEM: [u8; 16] = with_runs::<16>(&[
    (0, b"`\xa7\xeaDh\xcb\xcf\x11\x9cN"),
    (11, b"\xa0\xc9\x05B^"),
]);

/// `DPAID_ServiceProvider`: never written.
pub(super) const AID_SERVICE_PROVIDER: [u8; 16] = with_runs::<16>(&[
    (0, b"\xc0\x16\xd9\x07\xaf\xe0\xcf\x11\x9cN"),
    (11, b"\xa0\xc9\x05B^"),
]);

/// `DPAID_INet`: never written.
pub(super) const AID_INET: [u8; 16] = with_runs::<16>(&[
    (0, b"\xa0M\xa5\xc4\xaf\xe0\xcf\x11\x9cN"),
    (11, b"\xa0\xc9\x05B^"),
]);

/// `DPAID_INetPort`: never written.
pub(super) const AID_INET_PORT: [u8; 16] = with_runs::<16>(&[
    (0, b"AER\xe4\xa5\x8e\xd1\x11\x8a\x96"),
    (11, b"`\x97\xb0\x14\x11"),
]);

/// "susanivanovafanclub", the host's group's name; never written.
pub(super) const GROUP_NAME: [u8; 20] = with_runs::<20>(&[(0, b"susanivanovafanclub")]);

/// A player's 16 bytes of DirectPlay player data (`NetPlayerRecord`; this machine's own record
/// is `own`).
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct PlayerRecord {
    /// +0 the state bits (1 ready, 2 waiting, 4 racing, 0x10 the host), +1 the car, +2 its
    /// colours, +3 the nationality, +4 the stage, +5 its direction, +6 the race number, +7 a
    /// random number.
    pub head: Bytes<8>,
    /// The race's time.
    pub race_time: Dword,
    /// The total.
    pub total: Dword,
}

impl PlayerRecord {
    pub const fn zero() -> Self {
        Self {
            head: Bytes::new([0; 8]),
            race_time: Dword::new(0),
            total: Dword::new(0),
        }
    }

    /// The player's state.
    pub fn state(&self) -> State {
        State::from_byte(self.head.get(record::STATE))
    }

    pub(super) fn set_state(&self, state: State) {
        self.head.set(record::STATE, state.byte());
    }

    /// The record as a value.
    pub fn get(&self) -> Record {
        Record::from_bytes(&self.bytes())
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

/// A player's block: name, id, record and the copy of the messages taken for the race. The
/// messages themselves wait in the player's `Inbox`, behind the received messages' lock.
#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ByteView)]
pub struct PlayerBlock {
    /// The player's name.
    pub name: Bytes<32>,
    /// Its DirectPlay id.
    pub id: Dword,
    /// Its record.
    pub record: PlayerRecord,
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
            copied_count: Dword::new(0),
            copied: Bytes::new([0; 0x4000]),
        }
    }
}

/// The players' inboxes: one per block, under one lock that also covers the chat's texts.
pub struct Inboxes(std::sync::LazyLock<std::sync::Mutex<[Inbox; 8]>>);

impl Inboxes {
    pub const fn new() -> Self {
        Self(std::sync::LazyLock::new(|| {
            std::sync::Mutex::new(std::array::from_fn(|_| Inbox {
                count: 0,
                records: vec![0; rc2k_core::net::inbox::RECORDS_LEN],
                oldest: 0,
                newest: 0,
                time: 0,
            }))
        }))
    }

    /// Takes the lock; it is given back when the guard is dropped.
    pub fn lock(&self) -> std::sync::MutexGuard<'_, [Inbox; 8]> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Player `n`'s inbox among the locked ones; a number of 8 or more is refused.
pub(super) fn inbox_of(
    inboxes: &mut [Inbox; 8],
    n: u32,
) -> Result<&mut Inbox, crate::errors::BadInput> {
    inboxes
        .get_mut(n as usize)
        .ok_or(crate::errors::BadInput("a player number of 8 or more"))
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

/// What the screen's text edit works on: the item edited, which text, how wide it may be, its
/// font, and the text as it was before the edit (put back by Escape). The network settings and
/// the lobby each have their own.
#[derive(rc2k_derive::Cells)]
pub struct Editing {
    /// The text of the item edited (none: nothing is edited).
    pub edit_text: crate::net::begin_edit::EditedTextCell,
    /// The item edited (-1: none).
    pub edited: PlainDword,
    /// The widest the edited text may be.
    pub edit_width: PlainDword,
    /// The font the edited text is measured with.
    pub edit_font: crate::gfx::typefaces::FontCell,
    /// The edited text as it was before the edit, put back by Escape.
    pub text_before_edit: PlainBytes<64>,
}

impl Editing {
    pub const fn new() -> Self {
        Self {
            edit_text: crate::net::begin_edit::EditedTextCell::new(),
            edited: PlainDword::new(0),
            edit_width: PlainDword::new(0),
            // 0 in the image: the first font's number; read only once an edit has set it.
            edit_font: crate::gfx::typefaces::FontCell::new(
                crate::gfx::typefaces::FontId::numbered(0),
            ),
            text_before_edit: PlainBytes::new([0; 64]),
        }
    }
}

/// The network settings screen's state, the game thread's.
#[derive(rc2k_derive::Cells)]
pub struct SettingsScreen {
    /// The setting being edited.
    pub edit: Editing,
    /// The players' count as text while it is edited.
    pub most_players_text: PlainBytes<64>,
    /// The saved connection's name.
    pub saved_name: PlainBytes<64>,
}

impl SettingsScreen {
    pub const fn new() -> Self {
        Self {
            edit: Editing::new(),
            most_players_text: PlainBytes::new([0; 64]),
            saved_name: PlainBytes::new([0; 64]),
        }
    }
}

/// The sessions list screen's state, the game thread's.
#[derive(rc2k_derive::Cells)]
pub struct SessionsScreen {
    /// The time of the sessions' last listing.
    pub listed_at: PlainDword,
}

impl SessionsScreen {
    pub const fn new() -> Self {
        Self {
            listed_at: PlainDword::new(0),
        }
    }
}

/// This machine's network settings and name: the fields kept in the options file, the
/// player's name and a hosted session's mark.
#[derive(rc2k_derive::Cells)]
pub struct SetupVars {
    /// This player's name.
    pub player_name: PlainBytes<32>,
    /// The machine's name (kept in `var\ral.dat`).
    pub machine_name: PlainBytes<64>,
    /// The most players a hosted session takes (kept).
    pub most_players: PlainDword,
    /// The four values of the game's own in a hosted session's description: a fixed mark
    /// (the second's bit 0 says the session is this game's). Nobody types them and nothing
    /// compares them as a password.
    pub session_user_values: PlainBytes<16>,
    /// The host's address (TCP/IP), kept.
    pub host_address: PlainBytes<64>,
    /// The TCP/IP port (the `inetport` setting, kept).
    pub port: PlainDword,
    /// The saved connection's GUID (kept).
    pub saved_connection: PlainBytes<16>,
}

impl SetupVars {
    pub const fn new() -> Self {
        Self {
            player_name: PlainBytes::new([0; 32]),
            machine_name: PlainBytes::new([0; 64]),
            most_players: PlainDword::new(0x8),
            session_user_values: PlainBytes::new(with_runs::<16>(&[(0, b"\x17\x00\x00\x00\x01")])),
            host_address: PlainBytes::new([0; 64]),
            port: PlainDword::new(0),
            saved_connection: PlainBytes::new([0; 16]),
        }
    }
}

/// The address being built for a connection, the game thread's.
#[derive(rc2k_derive::Cells)]
pub struct AddressVars {
    /// The address elements added.
    pub elements: PlainDword,
    /// The compound address (0: it is kept in `net::link::Link`'s tables).
    pub address: PlainDword,
    /// The compound address's size.
    pub address_size: PlainDword,
}

impl AddressVars {
    pub const fn new() -> Self {
        Self {
            elements: PlainDword::new(0),
            address: PlainDword::new(0),
            address_size: PlainDword::new(0),
        }
    }
}

/// The joined variables, the game thread's.
#[derive(rc2k_derive::Cells)]
pub struct JoinedVars {
    /// The joined session's GUID.
    pub joined: PlainBytes<16>,
    /// Its name.
    pub joined_name: PlainBytes<64>,
    /// Its most players.
    pub joined_most: PlainDword,
    /// Its description's four values of the game's own (the host's mark).
    pub joined_user_values: PlainBytes<16>,
}

impl JoinedVars {
    pub const fn new() -> Self {
        Self {
            joined: PlainBytes::new([0; 16]),
            joined_name: PlainBytes::new([0; 64]),
            joined_most: PlainDword::new(0),
            joined_user_values: PlainBytes::new([0; 16]),
        }
    }
}

/// The lobby screen's state, the game thread's.
#[derive(rc2k_derive::Cells)]
pub struct LobbyScreen {
    /// The chat message being edited.
    pub edit: Editing,
    /// What this player types into the chat.
    pub chat_input: PlainBytes<64>,
    /// The time of the players' last listing.
    pub listed_at: PlainDword,
    /// The lobby's car item can be chosen.
    pub car_item: PlainDword,
    /// 0 the settings, 1 host, 2 join, 3 lobby.
    pub mode: PlainDword,
    /// The first player not the host.
    pub first_host: PlainDword,
    /// The message being sent.
    pub send_buffer: PlainBytes<256>,
    /// Times shown as gaps.
    pub show_gaps: PlainDword,
    /// The size of the player data listed.
    pub data_size: PlainDword,
    /// Back in the lobby after a race.
    pub after_race: PlainDword,
    /// The players are not listed again.
    pub racing: PlainDword,
    /// The time of the last sync message.
    pub sync_clock: PlainDword,
    /// The sequence byte of the messages sent.
    pub sequence: PlainByte,
    /// 1 when the last network race's replay is kept and can be saved from the lobby.
    pub replay_kept: PlainByte,
    /// Two bytes that fill the record's last dword (`Cells` takes no padding); nothing
    /// reads or writes them.
    pub _pad_cells: PlainBytes<2>,
}

impl LobbyScreen {
    pub const fn new() -> Self {
        Self {
            edit: Editing::new(),
            chat_input: PlainBytes::new([0; 64]),
            listed_at: PlainDword::new(0),
            car_item: PlainDword::new(0x1),
            mode: PlainDword::new(0x3),
            first_host: PlainDword::new(0),
            send_buffer: PlainBytes::new([0; 256]),
            show_gaps: PlainDword::new(0),
            data_size: PlainDword::new(0),
            after_race: PlainDword::new(0),
            racing: PlainDword::new(0),
            sync_clock: PlainDword::new(0),
            sequence: PlainByte::new(128),
            replay_kept: PlainByte::new(0),
            _pad_cells: PlainBytes::new([0; 2]),
        }
    }
}

/// The network's variables only the game's thread reaches, in plain cells: this machine's
/// settings, the address being built, the session joined and the state of each screen. The
/// receiving thread reads and writes none of them (it works on `NetShared`).
pub struct NetMain {
    pub setup: SetupVars,
    pub address: AddressVars,
    pub joined: JoinedVars,
    pub settings: SettingsScreen,
    pub sessions: SessionsScreen,
    pub lobby: LobbyScreen,
}

impl NetMain {
    pub const fn new() -> Self {
        NetMain {
            setup: SetupVars::new(),
            address: AddressVars::new(),
            joined: JoinedVars::new(),
            settings: SettingsScreen::new(),
            sessions: SessionsScreen::new(),
            lobby: LobbyScreen::new(),
        }
    }
}

/// What the receiving thread works with and hands to the game thread.
#[derive(rc2k_derive::Cells)]
pub struct ReceiverState {
    /// The receiving thread is busy.
    pub receiving: Dword,
    /// The messages the receiving thread took, counted.
    pub messages_taken: Dword,
    /// The sync round, counted up at each lobby.
    pub sync: Byte,
    /// One byte that aligns `sync_answer` (`Cells` takes no padding); nothing reads or writes it.
    pub _pad_cells: Bytes<1>,
    /// The race-sync answer sent (0x41 or 0x51, the round).
    pub sync_answer: Word,
    /// This machine has joined a session and not yet heard the host's round. Every machine
    /// counts the rounds up as it enters the lobby, so one that joins after races (the host
    /// lets players in again between them, `net::enable_new_players`) is behind the others:
    /// it takes the round of the host's first call.
    pub round_unknown: Flag,
    /// The bytes of the message `net::receive_message` took.
    pub receive_data: Bytes<256>,
    /// The sender of the message taken.
    pub received_from: Dword,
    /// Its addressee.
    pub received_to: Dword,
    /// The size of the buffer / the message taken.
    pub received_size: Dword,
    /// DirectPlay's calls are made on the receiving thread.
    pub posting: Dword,
    /// The call posted to the receiving thread.
    pub posted_call: Dword,
    /// Its result.
    pub posted_result: Dword,
    /// The receiving thread has answered.
    pub posted_answered: Dword,
}

impl ReceiverState {
    pub const fn new() -> Self {
        Self {
            receiving: Dword::new(0),
            messages_taken: Dword::new(0),
            sync: Byte::new(0),
            _pad_cells: Bytes::new([0; 1]),
            sync_answer: Word::new(0),
            round_unknown: Flag::new(false),
            receive_data: Bytes::new([0; 256]),
            received_from: Dword::new(0),
            received_to: Dword::new(0),
            received_size: Dword::new(0),
            posting: Dword::new(0),
            posted_call: Dword::new(0),
            posted_result: Dword::new(0),
            posted_answered: Dword::new(0),
        }
    }
}

/// The network's variables the receiving thread shares with the game thread (atomic cells).
#[derive(rc2k_derive::Cells)]
pub struct NetShared {
    /// Set when the receiving thread has ended; also the `nonetwork` setting's variable
    /// (`settings::read_settings`).
    pub receive_done: Dword,
    /// The chat's lines (0x300 bytes).
    pub chat: Bytes<768>,
    /// A chat line being cut.
    pub chat_line: Bytes<128>,
    /// A chat message "name> text".
    pub chat_text: Bytes<128>,
    /// What the receiving thread works with.
    pub receiver: ReceiverState,
    /// The receiving thread's event (a handle).
    pub event: Dword,
    /// The lobby's stage item can be chosen.
    pub stage_item: Dword,
    /// This machine's player record.
    pub own: PlayerRecord,
    /// Every player is ready.
    pub all_raced: Dword,
    /// Every player has finished.
    pub race_over: Dword,
    /// The sync table is not cleared yet.
    pub sync_first: Dword,
    /// Whether each player has answered the sync.
    pub synced: [Flag; 8],
    /// The players' blocks.
    pub received: [PlayerBlock; 8],
}

impl NetShared {
    pub const fn new() -> Self {
        NetShared {
            receive_done: Dword::new(0),
            chat: Bytes::new([0; 768]),
            chat_line: Bytes::new([0; 128]),
            chat_text: Bytes::new([0; 128]),
            receiver: ReceiverState::new(),
            event: Dword::new(0),
            stage_item: Dword::new(0x1),
            own: PlayerRecord::zero(),
            all_raced: Dword::new(0),
            race_over: Dword::new(0),
            sync_first: Dword::new(0x1),
            synced: [const { Flag::new(false) }; 8],
            received: [const { PlayerBlock::zero() }; 8],
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
    pub(super) fn player_block_or_end(&self, n: u32) -> &PlayerBlock {
        self.player_block(n)
            .unwrap_or_else(|e| crate::fatal::raise::end("a player's block", e))
    }

    /// Whether player `n` has answered the sync (the players are at most 8).
    pub(super) fn synced(&self, n: usize) -> Result<bool, crate::errors::BadInput> {
        Ok(self.synced_flag(n)?.is_set())
    }

    pub(super) fn set_synced(&self, n: usize, value: bool) -> Result<(), crate::errors::BadInput> {
        self.synced_flag(n)?.set(value);
        Ok(())
    }

    /// No player has answered.
    pub(super) fn clear_synced(&self) {
        for flag in &self.synced {
            flag.set(false);
        }
    }

    fn synced_flag(&self, n: usize) -> Result<&Flag, crate::errors::BadInput> {
        self.synced.get(n).ok_or(crate::errors::BadInput(
            "a player's synced flag past the eighth",
        ))
    }
}

/// `net::draw_button`'s arguments and measurements, the game thread's.
#[derive(rc2k_derive::Cells)]
pub struct Button {
    /// `net::draw_button`'s string.
    pub text: PlainDword,
    /// Its flags.
    pub flags: PlainDword,
    /// Its x.
    pub x: PlainDword,
    /// Its y.
    pub y: PlainDword,
    /// Its width.
    pub width: PlainDword,
    /// Its frame's tiles.
    pub tiles: PlainDword,
    /// Its item.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A record kept in the cells reads back as the value it was set from, in every state.
    #[test]
    fn a_player_record_in_cells_round_trips_in_every_state() {
        let cells = PlayerRecord::zero();
        for byte in [0u8, 1, 2, 4, 0x10, 0x11, 0x14, 0x82, 0xFF] {
            let record = Record {
                state: State::from_byte(byte),
                car: 3,
                colours: 4,
                nationality: 5,
                stage: 0x29,
                direction: 1,
                race_number: 6,
                random: 7,
                race_time: u32::MAX,
                total: 0x1234_5678,
            };
            cells.set_bytes(&record.to_bytes());
            assert_eq!(cells.get(), record);
            assert_eq!(cells.state(), State::from_byte(byte));
            assert_eq!(cells.bytes(), record.to_bytes());
        }
    }

    /// The sync flags are eight, set one by one and cleared together; a ninth is refused.
    #[test]
    fn the_sync_flags_are_eight() {
        let vars = NetShared::new();
        assert!(!vars.synced(7).unwrap());
        vars.set_synced(7, true).unwrap();
        assert!(vars.synced(7).unwrap() && !vars.synced(6).unwrap());
        vars.clear_synced();
        assert!(!vars.synced(7).unwrap());
        assert!(vars.synced(8).is_err() && vars.set_synced(8, true).is_err());
    }
}
