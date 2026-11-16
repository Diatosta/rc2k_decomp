//! `menu::player_to_standing_b`: a standings-B entry filled from a player record: +4 the
//! player's index + 1, +0x14 the name's 0x20 bytes (from the record's +0x18 pointer), +8 / +0xC
//! / +0x10 its +4 / +8 / +0x90, +0x34 its +0x8C, +0x38 .. +0x50 its +0xB4 .. +0xCC, +0x54 ..
//! +0x6C its +0x98 .. +0xB0.
//!
//! Refuses a null entry, record or name, before anything.
use crate::errors::Refused;
use crate::menu::standings::*;

/// `e` the entry of table B, `p` the player's record, `index` the player's index.
pub fn player_to_standing_b(
    menus: &crate::menu::owner::Menus,
    e: usize,
    p: PlayerCell,
    index: u32,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    name_player(&p.get())?;
    use rc2k_core::menu::standings::PlayerB;
    let source = PlayerB {
        index,
        name: name_of(&menus.play, &p.get())?,
        nationality: p.get().nationality,
        car: p.get().car_number,
        group: p.get().class,
        total: p.get().total,
        b4: std::array::from_fn(|k| p.get().overall_points[k]),
        slots: std::array::from_fn(|k| p.get().stage_points[k]),
    };
    with_b(tables, |t| {
        rc2k_core::saves::rankings::fill_b(&mut t.entries[e], &source);
    })?;
    Ok(())
}
