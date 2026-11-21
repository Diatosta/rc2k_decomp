//! `menu::player_to_standing_b`: a standings-B entry filled from a player record: `player` the
//! player's index + 1, `name` the name's 0x20 bytes (from the record's name pointer),
//! `nation` / `car` / `group` its nationality, car number and `class`, `total` its `total`,
//! `overall` its `overall_points`, `stage` its `stage_points`.
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
