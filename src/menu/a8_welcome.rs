//! The A8 championship's welcome: the picture `a8_welc.png` with the title bar
//! (`gfx::load_picture_with_overlay`; a failed load is fatal error 0x80712345, no message),
//! shown frame by frame (`screens::run_frame` with `menu::picture_frame`) until a key is taken
//! (`input::take_key`). Escape asks about leaving (`menu::confirm_leaving`); a no starts over.
//! Then one more key is taken and dropped.
//!
//! Caller: `menu::run_a8_championship` (event mode 0x13). The calls go through `event_flow::Call`.

use crate::errors::Refused;
use crate::menu::flow_game::Game;
use rc2k_core::menu::welcome::{self};

pub fn a8_welcome(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    Game::new(o).run(welcome::a8_welcome)
}
