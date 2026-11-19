//! The network game's way through (`net::start_link`, 1 host, 2 join): the player's record is set
//! up, connections are listed and one chosen (unless the saved one is there), the connection
//! made, then the session hosted or the sessions listed and one joined. Then the lobby screen
//! (`net::lobby_loop`) and the race (`net::lets_rally`) run until a player leaves. The settings
//! screen before all that is `net::settings_loop`.
//!
//! A player's record is 16 bytes of DirectPlay player data: the state bits (1 ready, 2 waiting,
//! 4 raced, 0x10 host), the car, its colours, the nationality, the stage, its direction, the race
//! number, a random number, the race's time and the total.

use crate::errors::Refused;
use crate::memory::session::Dword;

// ---- game data ----

pub const PALETTE: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::MEASURE_23;
pub const FONT: crate::gfx::typefaces::FontId = crate::gfx::typefaces::fonts::HEADING_23;

// ---- the list screens' variables (`menu::list_*`) ----

// ---- small pieces ----

/// `rep movsb` of `n` bytes, forwards (the places do not overlap).
pub fn movsb(
    to: impl Into<crate::memory::place::BytePlace>,
    from: impl Into<crate::memory::place::BytePlace>,
    n: u32,
) -> Result<(), Refused> {
    let bytes = from.into().read(n as usize, "a network record's bytes")?;
    to.into().write(&bytes, "a network record's bytes")?;
    Ok(())
}

/// `strings::append_string_by_id`: the string `id` added to the text being built.
pub fn push_id(
    o: crate::owners::Owners<'static>,
    text: &mut Vec<u8>,
    id: u32,
) -> Result<(), Refused> {
    crate::strings::append_string_by_id::push_string(
        crate::strings::append_string_by_id::StringOwners::of(o),
        text,
        id,
    )?;
    Ok(())
}

/// The text a line is built as (`text` up to its NUL), stored with its NUL at the start of
/// the game's text buffer, to be drawn or measured as a value.
pub fn store_text(
    strings: &crate::strings::owner::Strings,
    mut text: Vec<u8>,
) -> Result<Vec<u8>, Refused> {
    text.push(0);
    strings.text.buffer.text.write_at(0, &text, "a text line")?;
    Ok(text)
}

/// `gfx::blit_glyph` of the sprite `glyph` of the sprite table at (`x`, `y`). The sprites are
/// direct colour and never look at a palette.
pub fn blit(
    glyph: u32,
    x: u32,
    y: u32,
    gfx: &crate::gfx::owner::Gfx,
    menus: &crate::menu::owner::Menus,
) -> Result<(), Refused> {
    crate::gfx::blit_glyph::blit_glyph_on_surface(
        glyph,
        crate::gfx::blit_marker_pair::sprite_table(menus),
        x as i32,
        y as i32,
        None,
        gfx,
    )
}

/// A log line at the given level from a text in hand (up to its NUL or its end).
pub fn log_bytes(text: &[u8], newline: bool, level: u8) {
    let n = text.iter().position(|&c| c == 0).unwrap_or(text.len());
    crate::logfile::write_line::log_text(&text[..n], newline, level);
}

/// A hexadecimal number logged at level 2.
pub fn log_hex(value: u32, newline: bool) {
    crate::logfile::log_hex8::log_hex8(value, newline, 2);
}

/// A number logged at the given level, with a line end.
pub fn log_int_nl(value: u32, level: u8) {
    crate::logfile::log_int::log_int(value as i32, true, level);
}

/// `system::zero_memory`.
pub fn zero(at: impl Into<crate::memory::place::BytePlace>, length: u32) -> Result<(), Refused> {
    at.into()
        .write(&vec![0; length as usize], "memory to zero")?;
    Ok(())
}

/// `CoCreateInstance` (`net::dplay::create_instance`): the object into `out` (0 when none
/// is made), its result.
pub fn co_create(class: &[u8; 16], interface: &[u8; 16], out: &Dword) -> u32 {
    match crate::net::dplay::create_instance(class, interface) {
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
pub fn run_frame<T>(
    o: crate::owners::Owners<'static>,
    frame: impl FnOnce() -> Result<T, Refused>,
) -> Result<(), Refused> {
    match crate::platform::run_frame_now(o, frame) {
        Some(done) => done.map(|_| ()),
        None => Ok(()),
    }
}

/// `menu::list_take_key`: the key.
pub fn list_key(
    list_state: &crate::menu::list_state::ListState,
    input: &crate::input::owner::Input,
) -> Result<u8, Refused> {
    Ok(crate::menu::list_take_key::list_take_key(list_state, input))
}

/// A list screen's other keys: a screenshot (R, when they are on), the lexicon.
pub fn other_key(o: crate::owners::Owners<'static>, key: u8) -> Result<(), Refused> {
    if o.gfx.shots.on.get() != 0 && key == 0x52 {
        crate::gfx::take_screenshot::take_screenshot(o.gfx, o.system)?;
    }
    crate::language::maybe_compile_lexicon::maybe_compile_lexicon(
        o.input.devices.keys_down.get(0x4E),
        o.strings,
        &o.system.work().files,
    )
}

/// `gfx::lock_surface`: whether it is locked.
pub fn lock_surface(gfx: &crate::gfx::owner::Gfx, system: &crate::system::owner::System) -> bool {
    crate::platform::lock_surface(gfx, system) >= 1
}

/// `gfx::copy_picture_to_surface`.
pub fn copy_picture(
    gfx: &crate::gfx::owner::Gfx,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    {
        crate::gfx::copy_picture_to_surface::copy_picture_to_surface(
            &mut crate::gfx::copy_picture_to_surface::GameSurface(gfx, system),
        )
    }?;
    Ok(())
}

/// `text::draw_string_by_id` with the given `font`.
pub fn draw_string(
    so: crate::strings::append_string_by_id::StringOwners<'static>,
    id: u32,
    font: crate::gfx::typefaces::FontId,
    x: u32,
    y: u32,
    gfx: &'static crate::gfx::owner::Gfx,
) -> Result<(), Refused> {
    crate::text::draw_string_by_id::draw_string_by_id_at(
        so,
        id,
        font,
        x as i32,
        y as i32,
        &mut crate::text::draw_text::GameCanvas::new(gfx),
    )?;
    Ok(())
}

/// A fatal error with no message.
pub const fn fatal(code: u32) -> crate::errors::FatalError {
    crate::errors::FatalError { code, message: "" }
}

// ---- the session layer's callbacks ----

/// Runs a callback where the session layer calls it and gives its answer: a refused input
/// ends the program with its reason, a fatal error is raised (and answers 0 if it comes back).
pub fn callback_entry(name: &str, rust: impl FnOnce() -> Result<u32, Refused>) -> u32 {
    match rust() {
        Ok(answer) => answer,
        Err(e) => {
            crate::fatal::raise::refused(name, e);
            0
        }
    }
}
