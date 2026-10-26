//! `menu::a8_welcome` (0x47F3C7): the A8 championship's welcome: the picture
//! `a8_welc.png` with the title bar (`gfx::load_picture_with_overlay`; not read: fatal
//! error 0x80712345, no message), converted; frames of it (`screens::run_frame` with
//! `menu::picture_frame`) until a key (`input::take_key`). Escape (1) asks about leaving
//! (`menu::confirm_leaving`): a no starts over. Then one more key taken (dropped).
//!
//! In:  nothing
//! Out: every register preserved (`pusha` .. `popa`)
//!
//! Caller: `menu::run_a8_championship` (whose `g_eventMode` 0x13 picks it). The calls through
//! `event_flow::Call`.

use crate::errors::Refused;
use crate::menu::event_flow::Flow;
use crate::menu::flow_game::Game;
use rc2k_core::menu::welcome::{self};

/// The Rust version. `f` the registers the caller carries (the welcome is a flow of its
/// own: what it changes is its own, the original keeps every register).
pub fn a8_welcome(f: &Flow) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::welcome::a8_welcome`.
    Game::nested(f).run(welcome::a8_welcome)
}
