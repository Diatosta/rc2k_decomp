//! `menu::find_section_player` (0x4A940E): the first of the four players whose entry for
//! the section (`section`, +0xB4 + 4 * section) is 0x14 (a reading: done with it); none:
//! ecx -1.
//!
//! In:  nothing (`section`)
//! Out: eax the section, ecx the player (or -1), edi the last player looked at
//!
//! Callers: `sub_4A14CB`, `sub_4A9439`.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::menu::standings::*;

/// What `find_section_player` finds, as the original leaves it in the registers: ecx the player's index (`u32::MAX` for none), edi its record (player 4's for
/// none).
#[derive(Clone, Copy)]
pub struct SectionPlayer {
    pub index: u32,
    pub record: PlayerCell,
}

/// The Rust version (see the header).
pub fn find_section_player() -> Result<SectionPlayer, Refused> {
    // Converted: `rc2k_core::menu::points::section_player`.
    let section: u32 = menus().screens.event.section.get();
    let done: [u32; 4] =
        std::array::from_fn(|n| player(n as u32).get().overall_points[section as usize]);
    let found = rc2k_core::menu::points::section_player(done);
    Ok(SectionPlayer {
        index: found.unwrap_or(u32::MAX),
        record: player(found.unwrap_or(3)),
    })
}
