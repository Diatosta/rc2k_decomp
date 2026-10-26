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
use crate::memory::blocks::Place;
use crate::menu::event_flow::Fault;
use crate::menu::standings::*;

/// `e` the entry (edi in the original), `p` the player's record (esi), `index` the
/// player's index (ebx).
pub fn player_to_standing_b(e: Place, p: u32, index: u32) -> Result<(), Refused> {
    if p == 0 || get(p + 0x18)? == 0 {
        return Err(BadInput("a null record or name").into());
    }
    // Converted: `rc2k_core::menu::standings::fill_b`.
    use rc2k_core::menu::standings::{PlayerB, fill_b};
    let fault = Fault::default();
    let source = PlayerB {
        index,
        name: bytes(get(p + 0x18)?)?,
        nationality: get(p + 4)?,
        car: get(p + 8)?,
        group: get(p + 0x90)?,
        total: get(p + 0x8C)?,
        b4: std::array::from_fn(|k| fault.get(p + 0xB4 + 4 * k as u32)),
        slots: std::array::from_fn(|k| fault.get(p + 0x98 + 4 * k as u32)),
    };
    fault.check()?;
    // The entry's bytes as far as it is written (to +0x70).
    with_table(e, 0x70, 1, |t| fill_b(t, 0, &source))?;
    Ok(())
}
