//! `menu::player_to_standing_b` (0x46618A): a standings-B entry (edi) filled from the player
//! record esi: +4 ebx + 1, +0x14 the name's 0x20 bytes (from the record's +0x18 pointer),
//! +8 / +0xC / +0x10 its +4 / +8 / +0x90, +0x34 its +0x8C, +0x38 .. +0x50 its +0xB4 ..
//! +0xCC, +0x54 .. +0x6C its +0x98 .. +0xB0.
//!
//! In:  edi the entry, esi the record, ebx its index
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Caller: `sub_4A44A0` (three times). Refuses a null entry, record or name, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::event_flow::Fault;
use crate::menu::standings::*;

/// `e` the entry of table B (edi in the original), `p` the player's record (esi), `index`
/// the player's index (ebx).
pub fn player_to_standing_b(e: usize, p: PlayerCell, index: u32) -> Result<(), Refused> {
    if p.get().word(0x18) == 0 {
        return Err(BadInput("a null name").into());
    }
    // Converted: `rc2k_core::saves::rankings::fill_b`.
    use rc2k_core::menu::standings::PlayerB;
    let fault = Fault::default();
    let source = PlayerB {
        index,
        name: bytes(p.get().word(0x18))?,
        nationality: p.get().word(4),
        car: p.get().word(8),
        group: p.get().word(0x90),
        total: p.get().word(0x8C),
        b4: std::array::from_fn(|k| p.get().word(0xB4 + 4 * k)),
        slots: std::array::from_fn(|k| p.get().word(0x98 + 4 * k)),
    };
    fault.check()?;
    with_b(|t| rc2k_core::saves::rankings::fill_b(&mut t.entries[e], &source))?;
    Ok(())
}
