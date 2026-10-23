//! `menu::find_section_player` (0x4A940E): the first of the four players whose entry for
//! the section (`section`, +0xB4 + 4 * section) is 0x14 (a reading: done with it); none:
//! ecx -1.
//!
//! In:  nothing (`section`)
//! Out: eax the section, ecx the player (or -1), edi the last player looked at
//!
//! Callers: `sub_4A14CB`, `sub_4A9439`. Refuses a missing player, before anything.
//! Self-test: the original and the Rust version on players of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::framework::{Regs, game_read, rva};
use crate::menu::standings::*;

const SECTION_RVA: usize = rva(0x75595C); // section
/// The Rust version (see the header): the registers it leaves.
pub unsafe fn find_section_player(entry: &Regs) -> Result<Regs, Refused> {
    let mut g = *entry;
    if (0..4).any(|n| player(n) == 0) {
        return Err(BadInput("a missing player").into());
    }
    // Converted: `rc2k_core::menu::points::section_player`.
    g.eax = unsafe { game_read(SECTION_RVA) };
    let done: [u32; 4] = std::array::from_fn(|n| {
        get(player(n as u32)
            .wrapping_add(g.eax.wrapping_mul(4))
            .wrapping_add(0xB4))
    });
    let found = rc2k_core::menu::points::section_player(done);
    g.edi = player(found.unwrap_or(3));
    g.ecx = found.unwrap_or(u32::MAX);
    Ok(g)
}
