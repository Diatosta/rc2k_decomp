//! `menu::show_loading_picture` (0x47CEA8): shows the picture "loading".
//!
//! In:  nothing
//! Out: as `gfx::show_picture_file` leaves the registers (eax = 1, or the path if the
//!      file could not be read)
//! `gfx::show_picture_file` on `files\g\graphics\screens\loading.pcx`.
//! Fifteen callers, `menu::set_up_front_end` among them.
//! Shadow mode runs the original alone (it shows frames).
//! Self-test: a redirected copy whose call notes the path.

use crate::framework::{game_addr, rva};
use crate::gfx::show_picture_file::show_picture_file;

// ---- game data ----

const PATH_RVA: usize = rva(0x731BBF); // "files\g\graphics\screens\loading.pcx"

/// The picture shown; what `gfx::show_picture_file` returns (eax: 1, or the path).
pub fn show_loading_picture() -> Result<u32, crate::errors::Refused> {
    unsafe { show_picture_file(game_addr(PATH_RVA) as *mut u8) }
}
