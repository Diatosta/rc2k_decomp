//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A0E60` (twice). Refuses what the finding refuses, before anything.

use crate::errors::Refused;
use crate::menu::finish_calls::Game;
use rc2k_core::menu::finish;

/// The Rust version (see the header).
pub fn section_finish() -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::finish::section_finish`.
    Game::run(finish::section_finish)
}
