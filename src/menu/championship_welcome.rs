//! `menu::championship_welcome` (0x47F314): the championship's welcome: unless a saved
//! game goes on (`dword_755970` 1), the speech `;3files\%speech\rally\rally0.wav` started
//! (`sound::start_stream`, ecx `dword_755698`) and the commands waited for;
//! `dword_75555D`, `dword_755561` and `g_yesNoAnswer` 0; the picture `mob_welc.png` with
//! the title bar (`gfx::load_picture_with_overlay`; not read: fatal error 0x80712345, no
//! message), converted; a key taken (dropped); one frame of it (`screens::run_frame`
//! with `menu::picture_frame`); then three seconds for a key (`system::wait_or_key`), and
//! without one, keys taken until one comes while the speech plays (`sound::stream_playing`)
//! and the game is not quitting (`g_quitting`): the speech's end or the quitting end it
//! there. A key other than Escape (1) stops the speech and ends it; Escape stops it and
//! asks about leaving (`menu::confirm_leaving`): a yes stops it again and ends, a no
//! starts over.
//!
//! Caller: `menu::run_championship` (`g_eventMode` 2).

use crate::errors::Refused;
use crate::menu::flow_game::Game;
use rc2k_core::menu::welcome::{self};

/// The Rust version.
pub fn championship_welcome(menus: &crate::menu::owner::Menus) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::welcome::championship_welcome`.
    Game::new(menus).run(welcome::championship_welcome)
}
