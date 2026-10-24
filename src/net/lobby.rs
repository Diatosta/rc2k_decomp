//! The way through (`net::start_link`, 0x416F50, eax 1 host, 2 join, 3 launched by a lobby):
//! the player's record set up (`net::take_player_settings`, `net::fill_own_player`), the
//! DirectPlay connections listed (`net::find_connections`: the lobby object, DirectPlay,
//! `EnumConnections` into the 0x54-byte records at `dword_52AC38`), one chosen
//! (`net::choose_connection`, a list screen) unless the one saved (`unk_52AC94`) is there,
//! the connection made (`net::init_connection`), then hosting (`net::host_session`) or the
//! sessions listed (`net::choose_session`, `EnumSessions` into the 0x68-byte records at
//! `dword_52ACF8`) and one joined (`net::join_session`). Then the lobby screen
//! (`net::lobby_loop`: chat, car, stage, ready) and the race (`net::lets_rally`) until a
//! player leaves. The settings screen before all that is `net::settings_loop` (0x4168CF).
//! Each player's record is 16 bytes of DirectPlay player data (`byte_52AD64` for this
//! machine: +0 the state bits, 1 ready, 2 waiting, 4 racing, 0x10 the host; +1 the car, +2
//! its colours, +3 the nationality, +4 the stage, +5 its direction, +6 the race number,
//! +7 a random number, +8 the race's time, +0xC the total), read into each player's block
//! of `g_netReceived` (+0x20 the DirectPlay id, +0x24 the record).
//!
//! The calls by place: in the game the Rust versions, the logging, DirectPlay's `Send`
//! through the game (`net::game_part`), the screens' frames and the platform through the
//! game; DirectPlay's methods are called from Rust (`net::dplay`). In the tests: stand-ins
//! that note each call with its seven registers and answer what the test says, and test
//! COM objects whose methods note their arguments.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr};

pub use crate::net::{a, g, p};

// ---- game data (IDA addresses) ----

pub const SESSION: usize = 0x52A910; // g_netSession: 1 while a network game runs
pub const PLAYER: usize = 0x52A920; // g_netPlayer: this machine's player
pub const PLAYERS: usize = 0x52AD98; // g_netPlayers
pub const RECEIVED: usize = 0x52AD9C; // g_netReceived: 0x804C bytes a player
pub const BLOCK: u32 = std::mem::size_of::<crate::net::types::NetPlayerBlock>() as u32;
pub const MODE: usize = 0x52A908; // dword_52A908: 0 the settings, 1 host, 2 join, 3 lobby
pub const SYNC: usize = 0x52A914; // byte_52A914: counted up at each lobby
pub const SEQUENCE: usize = 0x52A915; // g_netSequence
pub const FIRST_HOST: usize = 0x52A924; // dword_52A924: the first player not the host
pub const OWN: usize = 0x52AD64; // byte_52AD64: this machine's player record, 16 bytes
pub const OWN_TIME: usize = 0x52AD6C; // dword_52AD6C
pub const OWN_TOTAL: usize = 0x52AD70; // dword_52AD70
pub const ALL_RACED: usize = 0x52AD78; // dword_52AD78: every player is ready
pub const RACE_OVER: usize = 0x52AD7C; // dword_52AD7C: every player has finished
pub const BACK_FROM_RACE: usize = 0x52AD80; // dword_52AD80
pub const RACING: usize = 0x52AD84; // dword_52AD84: the players are not listed again
pub const SYNC_CLOCK: usize = 0x52AD88; // dword_52AD88
pub const SYNC_FIRST: usize = 0x52AD8C; // dword_52AD8C: the sync table not cleared yet
pub const SYNCED: usize = 0x52AD90; // byte_52AD90: 8 bytes, 1 a player answered
pub const CLOCK_A: usize = 0x52A4C4; // dword_52A4C4: the players' last listing
pub const RECEIVE_DONE: usize = 0x52A4B0; // g_netReceiveDone
pub const CHAT: usize = 0x52A508; // byte_52A508: the chat's lines
pub const CHAT_LINE: usize = 0x52A808; // byte_52A808: a line being cut
pub const CHAT_TEXT: usize = 0x52A888; // word_52A888: "name> text"
pub const CHAT_INPUT: usize = 0x52A4C8; // byte_52A4C8: what this player types
pub const EDITED: usize = 0x52A4B8; // dword_52A4B8: the item edited (-1: none)
pub const EDIT_TEXT: usize = 0x52A4B4; // dword_52A4B4: its text
pub const EDIT_WIDTH: usize = 0x52A4BC; // dword_52A4BC: the widest it may be
pub const EDIT_FONT: usize = 0x52A4C0; // dword_52A4C0: the font it is measured with
pub const EDIT_SAVED: usize = 0x95EAF8; // byte_95EAF8: the text before editing, 64 bytes
pub const NUMBER_TEXT: usize = 0x95E8F8; // byte_95E8F8: a number edited as text
pub const MACHINE_NAME: usize = 0x52AB90; // g_machineName, 64 bytes
pub const HOST_ADDRESS: usize = 0x52ABE4; // unk_52ABE4: the host's address (TCP/IP), 64 bytes
pub const MOST_PLAYERS: usize = 0x52ABD0; // dword_52ABD0
pub const SESSION_PASSWORD: usize = 0x52ABD4; // unk_52ABD4, 16 bytes
pub const PLAYER_NAME: usize = 0x52AB70; // byte_52AB70, 0x20 bytes
pub const PORT: usize = 0x52AC24; // dword_52AC24
pub const CAR_ITEM: usize = 0x52AC28; // dword_52AC28: the car item can be chosen
pub const STAGE_ITEM: usize = 0x52AC2C; // dword_52AC2C: the stage item can be chosen
pub const SHOW_GAPS: usize = 0x52AC30; // dword_52AC30: times shown as gaps
pub const CONNECTIONS: usize = 0x52AC34; // dword_52AC34: connections listed
pub const CONNECTION_TABLE: usize = 0x52AC38; // dword_52AC38: 0x54 bytes each
pub const ELEMENTS: usize = 0x52AC3C; // dword_52AC3C: address elements
pub const ADDRESS: usize = 0x52AC40; // dword_52AC40: the compound address
pub const ADDRESS_SIZE: usize = 0x52AC44; // bytes
pub const ELEMENT_TABLE: usize = 0x52AC48; // unk_52AC48: 0x18 bytes each
pub const SAVED_CONNECTION: usize = 0x52AC94; // unk_52AC94: its guid, 16 bytes
pub const SAVED_NAME: usize = 0x52ACA4; // unk_52ACA4: its name, 64 bytes
pub const POSTING: usize = 0x52ACE4; // g_netPosting
pub const POSTED: usize = 0x52ACE8; // g_netPostedCall
pub const POSTED_RESULT: usize = 0x52ACEC; // g_netPostedResult
pub const POSTED_ANSWERED: usize = 0x52ACF0; // g_netPostedAnswered
pub const SESSIONS: usize = 0x52ACF4; // dword_52ACF4: sessions listed
pub const SESSION_TABLE: usize = 0x52ACF8; // dword_52ACF8: 0x68 bytes each
pub const SESSION_OPEN: usize = 0x52ACFC; // dword_52ACFC
pub const JOINED: usize = 0x52AD00; // unk_52AD00: the session's guid, 16 bytes
pub const JOINED_NAME: usize = 0x52AD10; // unk_52AD10, 64 bytes
pub const JOINED_MOST: usize = 0x52AD50; // dword_52AD50
pub const JOINED_PASSWORD: usize = 0x52AD54; // unk_52AD54, 16 bytes
pub const SORTED: usize = 0x56AFFC; // unk_56AFFC: the players sorted by time
pub const SEND_BUFFER: usize = 0x52AA68; // g_netSendBuffer: 0x100 bytes
pub const NET_MEMORY: usize = 0x52A928; // g_netMemory
pub const PLAYER_ID: usize = 0x5AB270; // g_netPlayerId
pub const SEND_TO: usize = 0x5AB274; // g_netSendTo: the group
pub const DPLAY_2: usize = 0x5AB284; // dword_5AB284
pub const DPLAY: usize = 0x5AB288; // g_dplay
pub const LOBBY: usize = 0x5AB28C; // g_dplayLobby
pub const LOBBY_LAUNCHED: usize = 0x5AB290; // g_lobbyLaunched
pub const SETTINGS: usize = 0x5AB294; // dword_5AB294: the connection settings
pub const SETTINGS_SIZE: usize = 0x5AB298; // dword_5AB298
pub const DP_NAME: usize = 0x5AB29C; // dword_5AB29C: a DPNAME, 0x10 bytes
pub const SESSION_DESC: usize = 0x5AB2AC; // dword_5AB2AC: a DPSESSIONDESC2, 0x50 bytes
pub const APP_GUID: usize = 0x5AB300; // dword_5AB300: the game's guid
pub const TEXT: usize = 0x93E430; // text: the game's text buffer
pub const SPRITES: usize = 0x755581; // g_spriteTable
pub const PALETTE: usize = 0x51E624; // palette: the measuring font
pub const FONT: usize = 0x51E6A4; // font

// ---- the list screens' variables (`menu::list_*`) ----

pub use crate::race::viewer::{CHOICE, CHOICE_BEFORE, FRAME_CLOCK, KEY_CLOCK, MOVE_CLOCK, gb, pb};

// ---- small pieces ----

/// The address of a player's block of `g_netReceived`.
pub fn block(player: u32) -> u32 {
    a(RECEIVED).wrapping_add(player.wrapping_mul(BLOCK))
}

/// `rep movsb` of `n` bytes, forwards (addresses).
pub fn movsb(to: u32, from: u32, n: u32) {
    for k in 0..n {
        unsafe { ((to + k) as *mut u8).write(((from + k) as *const u8).read()) };
    }
}

/// The address of a function or label of the game.
pub fn game(x: u32) -> u32 {
    game_addr(crate::framework::rva(x as usize)) as u32
}

/// A cursor into a text buffer: what the original's `ebx` is while it builds a line with
/// `strings::str_append`, `append_string_by_id`, `format_u32`, `append_time` and
/// `append_nul`.
#[derive(Clone, Copy)]
pub struct Cursor(pub u32);

impl Cursor {
    /// The start of the game's text buffer.
    pub fn text() -> Self {
        Self(a(TEXT))
    }

    /// `strings::str_append`: the NUL-ended text at `from`, its end the new cursor.
    pub fn string(&mut self, from: u32) {
        self.0 =
            unsafe { crate::strings::str_append::str_append(from as *const u8, self.0 as *mut u8) }
                as u32;
    }

    /// `strings::append_string_by_id`: the string `id`.
    pub fn string_id(&mut self, id: u32) {
        let mut out = self.0 as *mut u8;
        unsafe {
            crate::strings::append_string_by_id::append_string_by_id(id, &mut |from, n| {
                std::ptr::copy_nonoverlapping(from, out, n);
                out = out.add(n);
            });
        }
        self.0 = out as u32;
    }

    /// The bytes written at the cursor (`format_u32`'s, `append_time`'s).
    pub fn bytes(&mut self, bytes: &[u8]) {
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), self.0 as *mut u8, bytes.len()) };
        self.0 = self.0.wrapping_add(bytes.len() as u32);
    }

    /// `strings::append_nul`: a NUL written, the cursor past it.
    pub fn nul(&mut self) {
        unsafe { (self.0 as *mut u8).write(0) };
        self.0 = self.0.wrapping_add(1);
    }
}

/// `gfx::blit_glyph` of the sprite `glyph` of the sprite table (`g_spriteTable`) at (`x`,
/// `y`) with the palette at `palette`; the ebx it leaves (the next blit's palette in the
/// original, which hands each the one before leaves).
pub fn blit(glyph: u32, x: u32, y: u32, palette: u32) -> Result<u32, Refused> {
    let exit = unsafe {
        crate::gfx::blit_glyph::blit_glyph_on_surface(
            glyph,
            g(SPRITES) as *const u8,
            x as i32,
            y as i32,
            palette as *const u32,
        )
    }?;
    Ok(exit.ebx)
}

/// `logfile::log_write_l<level>[_nl]` on the text at the address `text`.
pub fn log(text: u32, newline: bool, level: u8) {
    unsafe { crate::logfile::log_write::log_write(text as *const u8, newline, level) };
}

/// `logfile::log_hex8_l2[_nl]`.
pub fn log_hex(value: u32, newline: bool) {
    crate::logfile::log_hex8::log_hex8(value, newline, 2);
}

/// `logfile::log_int_l<level>_nl`.
pub fn log_int_nl(value: u32, level: u8) {
    crate::logfile::log_int::log_int(value as i32, true, level);
}

/// `memory::allocate` on the network's list (`g_netMemory`): the memory's address.
pub fn net_allocate(bytes: u32) -> Result<u32, Refused> {
    Ok(unsafe { crate::memory::allocate::allocate(a(NET_MEMORY) as *mut _, bytes) }? as u32)
}

/// `strings::copy_printable` (addresses).
pub fn copy_printable(text: u32, to: u32, room: u32) -> Result<(), Refused> {
    unsafe {
        crate::strings::copy_printable::copy_printable(text as *const u8, to as *mut u8, room)
    }?;
    Ok(())
}

/// `system::zero_memory` (an address).
pub fn zero(at: u32, length: u32) -> Result<(), Refused> {
    unsafe { crate::system::zero_memory::zero_memory(at as *mut u8, length) }?;
    Ok(())
}

/// `CoCreateInstance` (`net::dplay::create_instance`): its result.
pub fn co_create(class: u32, interface: u32, out: u32) -> u32 {
    unsafe { crate::net::dplay::create_instance(class, interface, out) as u32 }
}

/// `screens::run_frame` with the frame `frame` (`platform::run_frame_now`: the window's
/// messages, then the frame unless the game is quitting).
pub fn run_frame<T>(frame: impl FnOnce() -> Result<T, Refused>) -> Result<(), Refused> {
    match crate::platform::run_frame_now(frame) {
        Some(done) => done.map(|_| ()),
        None => Ok(()),
    }
}

/// `menu::list_take_key`: the key.
pub fn list_key() -> Result<u8, Refused> {
    Ok(crate::menu::list_take_key::list_take_key().key)
}

/// A list screen's other keys: a screenshot (R, when they are on), the lexicon.
pub fn other_key(key: u8) -> Result<(), Refused> {
    if crate::gfx::shots::on() != 0 && key == 0x52 {
        crate::gfx::take_screenshot::take_screenshot()?;
    }
    unsafe { crate::strings::maybe_compile_lexicon::maybe_compile_lexicon() }
}

/// `gfx::lock_surface`: whether it is locked (the original's carry clear).
pub fn lock_surface() -> bool {
    crate::platform::lock_surface() >= 1
}

/// `gfx::copy_picture_to_surface`.
pub fn copy_picture() -> Result<(), Refused> {
    unsafe {
        crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
            &mut crate::gfx::copy_picture_to_surface::GameSurface,
        )
    }?;
    Ok(())
}

/// `text::draw_string_by_id` with the font at the IDA address `font`.
pub fn draw_string(id: u32, font: usize, x: u32, y: u32) -> Result<(), Refused> {
    unsafe {
        crate::text::draw_string_by_id::draw_string_by_id_at(
            id,
            a(font) as *mut _,
            x as i32,
            y as i32,
            &mut crate::text::draw_text::GameCanvas,
        )
    }?;
    Ok(())
}

/// `text::draw_text` with the font at the address `font`.
pub fn draw_text(font: u32, text: u32, x: u32, y: u32) -> Result<(), Refused> {
    unsafe {
        crate::text::draw_text::draw_text_at(
            font as *mut _,
            text as *const u8,
            x as i32,
            y as i32,
            &mut crate::text::draw_text::GameCanvas,
        )
    }?;
    Ok(())
}

/// A fatal error with no message.
pub const fn fatal(code: u32) -> crate::errors::FatalError {
    crate::errors::FatalError { code, message: "" }
}

pub use rc2k_core::net::lists::{NO_CONNECTION, NO_CONNECTIONS, NO_SESSION, NO_SESSIONS};
pub const CONNECTION_LEN: u32 = std::mem::size_of::<crate::net::types::NetConnection>() as u32;
pub const SESSION_LEN: u32 = std::mem::size_of::<crate::net::types::NetSession>() as u32;

/// Record `index` (unsigned) of a table of `len`-byte records at `table` holding `count`:
/// `jnb` the first fatal error, a null table the second.
pub fn record_of(
    index: u32,
    count: usize,
    table: usize,
    len: u32,
    codes: [u32; 2],
) -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::lists::record`.
    rc2k_core::net::lists::record(index, g(count), || g(table), len, codes)
        .map_err(|missing| fatal(missing.0).into())
}

/// A connection's record (`net::connection_entry`).
pub fn connection(index: u32) -> Result<u32, Refused> {
    record_of(
        index,
        CONNECTIONS,
        CONNECTION_TABLE,
        CONNECTION_LEN,
        [NO_CONNECTION, NO_CONNECTIONS],
    )
}

/// A session's record (`net::session_entry`).
pub fn session(index: u32) -> Result<u32, Refused> {
    record_of(
        index,
        SESSIONS,
        SESSION_TABLE,
        SESSION_LEN,
        [NO_SESSION, NO_SESSIONS],
    )
}

// ---- DirectPlay's callbacks (`stdcall`, their arguments on the stack) ----

/// A callback's Rust version: its arguments in, its eax out.
pub type Callback = fn(&[u32]) -> Result<u32, Refused>;

thread_local!(static CALLBACK_ARGS: std::cell::RefCell<(usize, Vec<u32>)> =
    const { std::cell::RefCell::new((0, Vec::new())) });

/// A callback's Rust version where DirectPlay calls it (2026-10-23): a refused
/// input ends the program with its reason (there is no game code to run in its place:
/// before 2026-10-23 it called the game's callback at `ral`, which in the standalone would
/// fault), a fatal error is raised.
pub fn callback_entry(name: &str, ral: u32, args: &[u32], rust: Callback) -> u32 {
    let result = rust(args);
    match result {
        Ok(eax) => eax,
        Err(Refused::BadInput(why)) => {
            crate::standalone::finish(&format!("{name} ({ral:#x}) refused: {}", why.0), 1)
        }
        Err(Refused::Fatal(error)) => {
            crate::framework::log(&format!(
                "{name}: the Rust version raises fatal error {:#x} {:?}",
                error.code, error.message
            ));
            let r = Regs::default();
            unsafe { crate::fatal::raise::raise(error, &r) };
            r.eax
        }
    }
}
