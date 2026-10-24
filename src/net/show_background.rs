//! `net::show_background` (0x4173E7): the network screens' background: "files\d\gfx\
//! chatzone.pcx" loaded (`gfx::load_picture`) and converted (`gfx::convert_picture`), the
//! picture's flags (`g_pictureFlags`) the screen mode's low twelve bits (`g_screenModeFlags`),
//! the picture made the surface (`gfx::picture_as_surface`), the screen's title drawn on it
//! (`text::draw_string_by_id` with `stru_51E944` at 0x316, 0x2F: 0x162 for the settings,
//! 0x14B hosting, 0x14C joining or launched; 0x15A once the session is on), copied to the
//! surface (`gfx::copy_picture_to_surface`) and shown (`gfx::show_frame`).
//!
//! In:  nothing
//! Out: nothing (every register kept, `pusha` .. `popa`; the flags read by no caller)
//! Callers: `net::settings_loop`, `net::start_link`, `net::stopping`, `net::lobby_loop`,
//! `net::choose_car`, `net::choose_stage`.

use crate::errors::Refused;
use crate::net::lobby::{MODE, SESSION, a, copy_picture, draw_string, g, p};

const PICTURE: usize = 0x5AB400; // "files\d\gfx\chatzone.pcx"
const SCREEN_MODE_FLAGS: usize = 0x4F4934; // g_screenModeFlags
const PICTURE_FLAGS: usize = 0x4F49CC; // g_pictureFlags
pub const TITLE_FONT: usize = 0x51E944; // stru_51E944

/// The Rust version.
pub fn show_background() -> Result<(), Refused> {
    unsafe {
        crate::gfx::load_picture::load_picture(a(PICTURE) as *mut u8)?;
        crate::gfx::convert_picture::convert_picture_now()?;
        p(PICTURE_FLAGS, g(SCREEN_MODE_FLAGS) & 0xFFF);
        crate::gfx::picture_as_surface::picture_as_surface()?;
    }
    let title = if g(SESSION) == 1 {
        0x15A
    } else {
        match g(MODE) {
            0 => 0x162,
            1 => 0x14B,
            _ => 0x14C,
        }
    };
    draw_string(title, TITLE_FONT, 0x316, 0x2F)?;
    copy_picture()?;
    crate::platform::show_frame_now();
    Ok(())
}
