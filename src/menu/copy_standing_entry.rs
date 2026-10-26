//! `menu::copy_standing_entry` (0x46613F): a standings entry (edi) filled from the player
//! record esi: +4 ebx + 1, +0x14 / +0x1C / +0x18 the record's +0x88 / +0x90 / +0x8C, +0xC
//! its name pointer (+0x18) and +0x20 the name's 0x20 bytes, +8 / +0x10 its +4 / +8, +0x4C
//! its +0xD0.
//!
//! In:  edi the entry, esi the record, ebx its index
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_4A2B75` and two of the game's. Refuses a null entry, record or name, before
//! anything.
use crate::errors::{BadInput, Refused};
use crate::menu::standings::*;

/// `e` the entry (edi in the original), `p` the player's record (esi), `index` the
/// player's index (ebx).
pub fn copy_standing_entry(e: u32, p: u32, index: u32) -> Result<(), Refused> {
    if e == 0 || p == 0 || get(p + 0x18)? == 0 {
        return Err(BadInput("a null entry, record or name").into());
    }
    // Converted: `rc2k_core::menu::standings::fill_a`.
    use rc2k_core::menu::standings::{PlayerA, fill_a};
    let name_at = get(p + 0x18)?;
    let source = PlayerA {
        index,
        time: get(p + 0x88)?,
        group: get(p + 0x90)?,
        total: get(p + 0x8C)?,
        name_at,
        name: bytes(name_at)?,
        nationality: get(p + 4)?,
        car: get(p + 8)?,
        entered: get(p + 0xD0)?,
    };
    // The entry's bytes as far as it is written (to +0x50).
    fill_a(&mut table(e, 0x50, 1)?, 0, &source);
    Ok(())
}
