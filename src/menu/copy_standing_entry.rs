//! `menu::copy_standing_entry` (0x46613F): a standings entry (edi) filled from the player
//! record esi: +4 ebx + 1, +0x14 / +0x1C / +0x18 the record's +0x88 / +0x90 / +0x8C, +0xC
//! its name pointer (+0x18) and +0x20 the name's 0x20 bytes, +8 / +0x10 its +4 / +8, +0x4C
//! its +0xD0.
//!
//! Callers: two of the game's. Refuses a null entry, record or name, before
//! anything.
use crate::errors::Refused;
use crate::menu::standings::*;

/// `e` the entry of table A (edi in the original), `p` the player's record (esi), `index`
/// the player's index (ebx).
pub fn copy_standing_entry(
    menus: &crate::menu::owner::Menus,
    e: usize,
    p: PlayerCell,
    index: u32,
) -> Result<(), Refused> {
    name_player(&p.get())?;
    // The logic is in `rc2k_core::saves::rankings::fill_a`.
    use rc2k_core::menu::standings::PlayerA;
    let name_at = name_address_of(&p.get())?;
    let source = PlayerA {
        index,
        time: p.get().time,
        group: p.get().class,
        total: p.get().total,
        name_at,
        name: name_of(menus, &p.get())?,
        nationality: p.get().nationality,
        car: p.get().car_number,
        entered: p.get().entry,
    };
    with_a(|t| rc2k_core::saves::rankings::fill_a(&mut t.entries[e], &source))?;
    Ok(())
}
