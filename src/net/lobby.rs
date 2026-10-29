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
use crate::framework::Regs;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::place::Place;
use crate::pointers::GamePtr;
use crate::session::{Bytes, Dword};
use crate::strings::text_buffer::TEXT_BUFFER;

// ---- game data (IDA addresses) ----

pub const SPRITES: &Dword = &LOOP_FLAGS.sprite_table; // g_spriteTable
pub const PALETTE: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::MEASURE_23;
pub const FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::HEADING_23;

// ---- the list screens' variables (`menu::list_*`) ----

// ---- small pieces ----

/// `rep movsb` of `n` bytes, forwards (addresses).
pub fn movsb(to: u32, from: u32, n: u32) -> Result<(), Refused> {
    GamePtr::new(from).copy_to(GamePtr::new(to), n as usize, "a network record's bytes")?;
    Ok(())
}

/// A cursor into a text buffer: what the original's `ebx` is while it builds a line with
/// `strings::str_append`, `append_string_by_id`, `format_u32`, `append_time` and
/// `append_nul`.
#[derive(Clone, Copy)]
pub struct Cursor(pub Place);

impl Cursor {
    /// The start of the game's text buffer.
    pub fn text() -> Self {
        Self(Place::from(&TEXT_BUFFER.text))
    }

    /// `strings::str_append`: the NUL-ended text at `from`, its end the new cursor.
    pub fn string(&mut self, from: impl Into<Place>) {
        let written = from
            .into()
            .text("a text")
            .and_then(|text| self.0.write(&text, "a text line").map(|()| text.len()));
        let len = crate::menu::event_flow::refuse_here(written, "net::lobby: a text");
        self.0 = self.0.add(len as u32);
    }

    /// `strings::append_string_by_id`: the string `id`.
    pub fn string_id(&mut self, id: u32) {
        let mut text = Vec::new();
        let written = crate::strings::append_string_by_id::push_string(&mut text, id)
            .and_then(|()| self.0.write(&text, "a text line"));
        crate::menu::event_flow::refuse_here(written, "net::lobby: a string");
        self.0 = self.0.add(text.len() as u32);
    }

    /// The bytes written at the cursor (`format_u32`'s, `append_time`'s).
    pub fn bytes(&mut self, bytes: &[u8]) -> Result<(), Refused> {
        self.0.write(bytes, "a text line's bytes")?;
        self.0 = self.0.add(bytes.len() as u32);
        Ok(())
    }

    /// `strings::append_nul`: a NUL written, the cursor past it.
    pub fn nul(&mut self) -> Result<(), Refused> {
        self.0.write(&[0], "a text line's end")?;
        self.0 = self.0.add(1);
        Ok(())
    }
}

/// `gfx::blit_glyph` of the sprite `glyph` of the sprite table (`g_spriteTable`) at (`x`,
/// `y`) with the palette at `palette`; the ebx it leaves (the next blit's palette in the
/// original, which hands each the one before leaves).
pub fn blit(glyph: u32, x: u32, y: u32, palette: u32) -> Result<u32, Refused> {
    let exit = {
        crate::gfx::blit_glyph::blit_glyph_on_surface(
            glyph,
            SPRITES.get(),
            x as i32,
            y as i32,
            palette,
        )
    }?;
    Ok(exit.ebx)
}

/// `logfile::log_write_l<level>[_nl]` on the text at the address `text`.
pub fn log(text: u32, newline: bool, level: u8) {
    crate::logfile::log_write::log_at(text, newline, level);
}

/// `log` of a text in hand (up to its NUL or its end).
pub fn log_bytes(text: &[u8], newline: bool, level: u8) {
    let n = text.iter().position(|&c| c == 0).unwrap_or(text.len());
    crate::logfile::write_line::log_text(&text[..n], newline, level);
}

/// `logfile::log_hex8_l2[_nl]`.
pub fn log_hex(value: u32, newline: bool) {
    crate::logfile::log_hex8::log_hex8(value, newline, 2);
}

/// `logfile::log_int_l<level>_nl`.
pub fn log_int_nl(value: u32, level: u8) {
    crate::logfile::log_int::log_int(value as i32, true, level);
}

/// The lobby's connection settings (`dword_5AB294`, `memory::allocate` on the network's
/// list in the original): Rust's bytes since phase 13 (2026-10-27), counted on the list as
/// the game's block was and lent to the platform, which fills them with DirectPlay's
/// structure and its own addresses inside; given back with the list (`free_messages`).
/// Only a lobby's launch reaches it, which the platform never reports
/// (`dp_objects::get_connection_settings`).
struct LobbySettings {
    // Declared first: the region goes before the bytes (`pointers::Lent`).
    _lent: crate::pointers::Lent,
    _bytes: Vec<u8>,
}

static LOBBY_SETTINGS: std::sync::Mutex<Option<LobbySettings>> = std::sync::Mutex::new(None);

/// `bytes` of settings taken: their game address.
pub fn take_lobby_settings(bytes: u32) -> Result<u32, Refused> {
    let memory = crate::memory::owned::counted(crate::net::tables::list(), bytes)?;
    let lent = crate::pointers::Lent::new(&memory);
    let at = lent.addr();
    *LOBBY_SETTINGS.lock().unwrap_or_else(|e| e.into_inner()) = Some(LobbySettings {
        _lent: lent,
        _bytes: memory,
    });
    Ok(at)
}

/// The settings given back (the list's `free_list` counts them down).
pub fn give_back_lobby_settings() {
    *LOBBY_SETTINGS.lock().unwrap_or_else(|e| e.into_inner()) = None;
}

/// `strings::copy_printable` (addresses).
pub fn copy_printable(text: u32, to: u32, room: u32) -> Result<(), Refused> {
    unsafe {
        crate::strings::copy_printable::copy_printable(
            crate::pointers::host(text) as *const u8,
            crate::pointers::host(to),
            room,
        )
    }?;
    Ok(())
}

/// `system::zero_memory` (an address).
pub fn zero(at: u32, length: u32) -> Result<(), Refused> {
    GamePtr::new(at).fill(0, length as usize, "memory to zero")?;
    Ok(())
}

/// `CoCreateInstance` (`net::dplay::create_instance`): the object into `out` (0 when none
/// is made), its result.
pub fn co_create(class: &Bytes<16>, interface: &Bytes<16>, out: &Dword) -> u32 {
    match crate::net::dplay::create_instance(&class.read(), &interface.read()) {
        Ok(made) => {
            out.set(made);
            0
        }
        Err(e) => {
            out.set(0);
            e
        }
    }
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
    {
        crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
            &mut crate::gfx::copy_picture_to_surface::GameSurface,
        )
    }?;
    Ok(())
}

/// `text::draw_string_by_id` with the font at the IDA address `font`.
pub fn draw_string(
    id: u32,
    font: crate::gfx::typefaces::FontId,
    x: u32,
    y: u32,
) -> Result<(), Refused> {
    unsafe {
        crate::text::draw_string_by_id::draw_string_by_id_at(
            id,
            font.as_ptr(),
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

// ---- DirectPlay's callbacks (`stdcall`, their arguments on the stack) ----

/// A callback's Rust version where DirectPlay calls it (2026-10-23): a refused
/// input ends the program with its reason (there is no game code to run in its place:
/// before 2026-10-23 it called the game's callback at `ral`, which in the standalone would
/// fault), a fatal error is raised. Its eax out.
pub fn callback_entry(name: &str, ral: u32, rust: impl FnOnce() -> Result<u32, Refused>) -> u32 {
    match rust() {
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
            crate::fatal::raise::raise(error, &r);
            r.eax
        }
    }
}
