//! `net::show_background` (0x4173E7): the network screens' background: "files\d\gfx\
//! chatzone.pcx" loaded (`gfx::load_picture`) and converted (`gfx::convert_picture`), the
//! picture's flags (`g_pictureFlags`) the screen mode's low twelve bits (`g_screenModeFlags`),
//! the picture made the surface (`gfx::picture_as_surface`), the screen's title drawn on it
//! (`text::draw_string_by_id` with `stru_51E944` at 0x316, 0x2F: 0x162 for the settings,
//! 0x14B hosting, 0x14C joining or launched; 0x15A once the session is on), copied to the
//! surface (`gfx::copy_picture_to_surface`) and shown (`gfx::show_frame`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//! Callers: `net::settings_loop`, `net::start_link`, `net::stopping`, `net::lobby_loop`,
//! `net::choose_car`, `net::choose_stage`. Its calls through `part` (`net::lobby_calls`: the
//! Rust versions, the frame shown through the game). Shadow mode runs the original alone (it
//! draws). Self-test: a copy whose calls go to stand-ins, against the Rust version, on each
//! mode with the session on and off.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{
    CONVERT_PICTURE, COPY_PICTURE, DRAW_STRING, LOAD_PICTURE, MODE, PICTURE_AS_SURFACE, Part,
    SESSION, SHOW_FRAME, a, g, lobby_entry, p,
};

const PICTURE: usize = 0x5AB400; // "files\d\gfx\chatzone.pcx"
const SCREEN_MODE_FLAGS: usize = 0x4F4934; // g_screenModeFlags
const PICTURE_FLAGS: usize = 0x4F49CC; // g_pictureFlags
pub const TITLE_FONT: usize = 0x51E944; // stru_51E944

/// The Rust version: the registers it leaves (the entry's).
pub fn show_background(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.eax = a(PICTURE);
    part(LOAD_PICTURE, &mut r)?;
    part(CONVERT_PICTURE, &mut r)?;
    r.eax = g(SCREEN_MODE_FLAGS) & 0xFFF;
    p(PICTURE_FLAGS, r.eax);
    part(PICTURE_AS_SURFACE, &mut r)?;
    r.eax = match g(MODE) {
        0 => 0x162,
        1 => 0x14B,
        _ => 0x14C,
    };
    if g(SESSION) == 1 {
        r.eax = 0x15A;
    }
    r.ebx = a(TITLE_FONT);
    r.ecx = 0x316;
    r.edx = 0x2F;
    part(DRAW_STRING, &mut r)?;
    part(COPY_PICTURE, &mut r)?;
    part(SHOW_FRAME, &mut r)?;
    Ok(Regs {
        eflags: r.eflags,
        ..*entry
    })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, show_background)
}
