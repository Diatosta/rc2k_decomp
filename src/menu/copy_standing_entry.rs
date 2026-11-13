//! `menu::copy_standing_entry`: a standings entry filled from a player record: +4 the player's
//! index + 1, +0x14 / +0x1C / +0x18 the record's +0x88 / +0x90 / +0x8C, +0xC its name pointer
//! (+0x18) and +0x20 the name's 0x20 bytes, +8 / +0x10 its +4 / +8, +0x4C its +0xD0.
//!
//! Callers: two of the game's. Refuses a null entry, record or name, before anything.
use crate::errors::Refused;
use crate::menu::standings::*;

/// `e` the entry of table A, `p` the player's record, `index` the player's index.
pub fn copy_standing_entry(
    menus: &crate::menu::owner::Menus,
    e: usize,
    p: PlayerCell,
    index: u32,
) -> Result<(), Refused> {
    name_player(&p.get())?;
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
