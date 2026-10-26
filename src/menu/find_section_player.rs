//! `menu::find_section_player` (0x4A940E): the first of the four players whose entry for
//! the section (`section`, +0xB4 + 4 * section) is 0x14 (a reading: done with it); none:
//! ecx -1.
//!
//! In:  nothing (`section`)
//! Out: eax the section, ecx the player (or -1), edi the last player looked at
//!
//! Callers: `sub_4A14CB`, `sub_4A9439`. Refuses a missing player, before anything.

use crate::errors::{BadInput, Refused};
use crate::menu::event_store::EVENT_STATE;
use crate::menu::standings::*;

/// What `find_section_player` finds, as the original leaves it in the registers: eax the
/// section, ecx the player's index (`u32::MAX` for none), edi its record (player 4's for
/// none).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectionPlayer {
    pub section: u32,
    pub index: u32,
    pub record: u32,
}

/// The Rust version (see the header).
pub fn find_section_player() -> Result<SectionPlayer, Refused> {
    if (0..4).any(|n| player(n) == 0) {
        return Err(BadInput("a missing player").into());
    }
    // Converted: `rc2k_core::menu::points::section_player`.
    let section: u32 = EVENT_STATE.section.get();
    let fault = crate::menu::event_flow::Fault::default();
    let done: [u32; 4] = std::array::from_fn(|n| {
        fault.get(
            player(n as u32)
                .wrapping_add(section.wrapping_mul(4))
                .wrapping_add(0xB4),
        )
    });
    fault.check()?;
    let found = rc2k_core::menu::points::section_player(done);
    Ok(SectionPlayer {
        section,
        index: found.unwrap_or(u32::MAX),
        record: player(found.unwrap_or(3)),
    })
}
