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
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `menu::run_championship` (`g_eventMode` 2). The calls through
//! `event_flow::Call` (in the game `game_call`: the Rust versions, the sound commands and
//! the frame through their hooks). Ungated (it waits). Shadow mode runs the original
//! alone. Self-test: `event_flow::test_flow`, keys and the speech by script.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::menu::event_flow::Flow;
use crate::menu::flow_game::Game;
use rc2k_core::menu::welcome::{self};

/// The Rust version. `f` the registers the caller carries (the welcome is a flow of its
/// own: what it changes is its own, the original keeps every register).
pub unsafe fn championship_welcome(f: &Flow) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::welcome::championship_welcome`.
    welcome::championship_welcome(&mut Game::nested(f))
}
