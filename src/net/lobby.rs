//! The way through (`net::start_link`, 1 host, 2 join):
//! the player's record set up (`net::take_player_settings`, `net::fill_own_player`), the
//! DirectPlay connections listed (`net::find_connections`: the lobby object, DirectPlay,
//! `EnumConnections` into the 0x54-byte records at `connection_table`), one chosen
//! (`net::choose_connection`, a list screen) unless the one saved (`saved_connection`) is there,
//! the connection made (`net::init_connection`), then hosting (`net::host_session`) or the
//! sessions listed (`net::choose_session`, `EnumSessions` into the 0x68-byte records at
//! `session_table`) and one joined (`net::join_session`). Then the lobby screen
//! (`net::lobby_loop`: chat, car, stage, ready) and the race (`net::lets_rally`) until a
//! player leaves. The settings screen before all that is `net::settings_loop`.
//! Each player's record is 16 bytes of DirectPlay player data (`own` for this
//! machine: +0 the state bits, 1 ready, 2 waiting, 4 racing, 0x10 the host; +1 the car, +2
//! its colours, +3 the nationality, +4 the stage, +5 its direction, +6 the race number,
//! +7 a random number, +8 the race's time, +0xC the total), read into each player's block
//! of `g_netReceived` (+0x20 the DirectPlay id, +0x24 the record).
//!
//! The calls by place: DirectPlay's methods are called from Rust (`net::dplay`). In the
//! tests: stand-ins that note each call and answer what the test says, and test COM
//! objects whose methods note their arguments.

use crate::errors::Refused;
use crate::session::PlainDword;
use crate::session::{Bytes, Dword};
use rc2k_core::fatal::Context;

// ---- game data (IDA addresses) ----

pub const SPRITES: &PlainDword = &crate::menu::owner::MENUS
    .unchecked()
    .screens
    .loop_flags
    .sprite_table; // g_spriteTable
pub const PALETTE: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::MEASURE_23;
pub const FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::HEADING_23;

// ---- the list screens' variables (`menu::list_*`) ----

// ---- small pieces ----

/// `rep movsb` of `n` bytes, forwards (the places do not overlap).
pub fn movsb(
    to: impl Into<crate::place::BytePlace>,
    from: impl Into<crate::place::BytePlace>,
    n: u32,
) -> Result<(), Refused> {
    let bytes = from.into().read(n as usize, "a network record's bytes")?;
    to.into().write(&bytes, "a network record's bytes")?;
    Ok(())
}

/// `strings::append_string_by_id`: the string `id` added to the text being built.
pub fn push_id(o: crate::owners::Owners<'_>, text: &mut Vec<u8>, id: u32) -> Result<(), Refused> {
    crate::strings::append_string_by_id::push_string_in(o, text, id)?;
    Ok(())
}

/// The text a line is built as (`text` up to its NUL), stored with its NUL at the start of
/// the game's text buffer, where the original builds it; the text with its NUL, to be
/// drawn or measured as a value.
pub fn store_text(
    strings: &crate::strings::owner::Strings,
    mut text: Vec<u8>,
) -> Result<Vec<u8>, Refused> {
    text.push(0);
    strings.text.buffer.text.write_at(0, &text, "a text line")?;
    Ok(text)
}

/// `gfx::blit_glyph` of the sprite `glyph` of the sprite table (`g_spriteTable`) at (`x`,
/// `y`). (The original hands each the palette the blit before left, which the sprites, direct
/// colour, never look at.)
pub fn blit(glyph: u32, x: u32, y: u32) -> Result<(), Refused> {
    crate::gfx::blit_glyph::blit_glyph_on_surface(glyph, SPRITES.get(), x as i32, y as i32, None)
}

/// `logfile::log_write_l<level>[_nl]` on a text in hand (up to its NUL or its end).
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

/// `system::zero_memory`.
pub fn zero(at: impl Into<crate::place::BytePlace>, length: u32) -> Result<(), Refused> {
    at.into()
        .write(&vec![0; length as usize], "memory to zero")?;
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
    Ok(crate::menu::list_take_key::list_take_key())
}

/// A list screen's other keys: a screenshot (R, when they are on), the lexicon.
pub fn other_key(o: crate::owners::Owners<'static>, key: u8) -> Result<(), Refused> {
    if crate::gfx::shots::on() != 0 && key == 0x52 {
        crate::gfx::take_screenshot::take_screenshot()?;
    }
    crate::strings::maybe_compile_lexicon::maybe_compile_lexicon(o.input, o.strings, o.system)
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
    crate::text::draw_string_by_id::draw_string_by_id_at(
        id,
        font,
        x as i32,
        y as i32,
        &mut crate::text::draw_text::GameCanvas,
    )?;
    Ok(())
}

/// A fatal error with no message.
pub const fn fatal(code: u32) -> crate::errors::FatalError {
    crate::errors::FatalError { code, message: "" }
}

// ---- DirectPlay's callbacks (`stdcall`, their arguments on the stack) ----

/// Runs a callback where DirectPlay calls it: a refused
/// input ends the program with its reason (there is no game code to run in its place), a
/// fatal error is raised.
pub fn callback_entry(name: &str, ral: u32, rust: impl FnOnce() -> Result<u32, Refused>) -> u32 {
    match rust() {
        Ok(eax) => eax,
        Err(Refused::BadInput(why)) => {
            crate::standalone::finish(&format!("{name} ({ral:#x}) refused: {}", why.0), 1)
        }
        Err(Refused::Fatal(error)) => {
            crate::framework::log(&format!(
                "{name}: raises fatal error {:#x} {:?}",
                error.code, error.message
            ));
            let r = Context::default();
            crate::fatal::raise::raise(error, &r);
            r.eax
        }
    }
}
