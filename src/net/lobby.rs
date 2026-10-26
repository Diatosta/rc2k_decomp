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
use crate::net::store::NET;
use crate::pointers::GamePtr;
use crate::session::Dword;
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

/// `rep movsb` from what DirectPlay's callbacks hand over (a GUID, a connection's data:
/// Rust's statics or buffers lent for the callback) to the game's memory; both checked.
pub fn movsb_from_rust(to: u32, from: u32, n: u32) -> Result<(), Refused> {
    let bytes = GamePtr::new(from).read_bytes(n as usize, "a network callback's bytes")?;
    GamePtr::new(to).write_bytes(&bytes, "a network record's bytes")?;
    Ok(())
}

/// A cursor into a text buffer: what the original's `ebx` is while it builds a line with
/// `strings::str_append`, `append_string_by_id`, `format_u32`, `append_time` and
/// `append_nul`.
#[derive(Clone, Copy)]
pub struct Cursor(pub u32);

impl Cursor {
    /// The start of the game's text buffer.
    pub fn text() -> Self {
        Self(TEXT_BUFFER.text.addr() as u32)
    }

    /// `strings::str_append`: the NUL-ended text at `from`, its end the new cursor.
    pub fn string(&mut self, from: u32) {
        self.0 = crate::menu::event_flow::refuse_here(
            crate::strings::str_append::str_append(from as *const u8, self.0 as *mut u8),
            "net::lobby: a text",
        ) as u32;
    }

    /// `strings::append_string_by_id`: the string `id`.
    pub fn string_id(&mut self, id: u32) {
        let mut text = Vec::new();
        let written = crate::strings::append_string_by_id::push_string(&mut text, id)
            .and_then(|()| GamePtr::new(self.0).write_bytes(&text, "a text line"));
        crate::menu::event_flow::refuse_here(written, "net::lobby: a string");
        self.0 = self.0.wrapping_add(text.len() as u32);
    }

    /// The bytes written at the cursor (`format_u32`'s, `append_time`'s).
    pub fn bytes(&mut self, bytes: &[u8]) -> Result<(), Refused> {
        GamePtr::new(self.0).write_bytes(bytes, "a text line's bytes")?;
        self.0 = self.0.wrapping_add(bytes.len() as u32);
        Ok(())
    }

    /// `strings::append_nul`: a NUL written, the cursor past it.
    pub fn nul(&mut self) -> Result<(), Refused> {
        GamePtr::new(self.0).write(0u8, "a text line's end")?;
        self.0 = self.0.wrapping_add(1);
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
            crate::pointers::GamePtr::new(SPRITES.get()),
            x as i32,
            y as i32,
            crate::pointers::GamePtr::new(palette),
        )
    }?;
    Ok(exit.ebx)
}

/// `logfile::log_write_l<level>[_nl]` on the text at the address `text`.
pub fn log(text: u32, newline: bool, level: u8) {
    crate::logfile::log_write::log_at(text, newline, level);
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
    Ok(
        { crate::memory::allocate::allocate((NET.memory.as_ptr() as u32) as *mut _, bytes) }?
            as u32,
    )
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
    GamePtr::new(at).fill(0, length as usize, "memory to zero")?;
    Ok(())
}

/// `CoCreateInstance` (`net::dplay::create_instance`): its result.
pub fn co_create(class: u32, interface: u32, out: u32) -> u32 {
    crate::net::dplay::create_instance(class, interface, out) as u32
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

/// `text::draw_text` with the font at the address `font`.
pub fn draw_text(font: u32, text: u32, x: u32, y: u32) -> Result<(), Refused> {
    {
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
    count: &Dword,
    table: &Dword,
    len: u32,
    codes: [u32; 2],
) -> Result<u32, Refused> {
    // Converted: `rc2k_core::net::lists::record`.
    rc2k_core::net::lists::record(index, count.get(), || table.get(), len, codes)
        .map_err(|missing| fatal(missing.0).into())
}

/// A connection's record (`net::connection_entry`).
pub fn connection(index: u32) -> Result<u32, Refused> {
    record_of(
        index,
        &NET.connections,
        &NET.connection_table,
        CONNECTION_LEN,
        [NO_CONNECTION, NO_CONNECTIONS],
    )
}

/// A session's record (`net::session_entry`).
pub fn session(index: u32) -> Result<u32, Refused> {
    record_of(
        index,
        &NET.sessions,
        &NET.session_table,
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
            crate::fatal::raise::raise(error, &r);
            r.eax
        }
    }
}
