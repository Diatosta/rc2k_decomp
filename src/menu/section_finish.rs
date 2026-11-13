//!
//! Refuses what the finding refuses, before anything.

use crate::errors::Refused;
use crate::menu::finish_calls::Game;
use rc2k_core::menu::finish;

pub fn section_finish(o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    Game::run(o, finish::section_finish)
}
