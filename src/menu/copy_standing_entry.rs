//! `menu::copy_standing_entry`: a standings entry filled from a player record: `player` the player's
//! index + 1, `key` / `group` / `rally_time` the record's `time` / `class` / `total`, `name_id`
//! its name pointer and `name` the name's 0x20 bytes, `nation` / `car` its nationality and car
//! number, `entered` its `entry`.
//!
//! Refuses a null entry, record or name, before anything.
use crate::errors::Refused;
use crate::menu::standings::*;

/// `e` the entry of table A, `p` the player's record, `index` the player's index.
pub fn copy_standing_entry(
    play: &crate::menu::owner::Play,
    e: usize,
    p: PlayerCell,
    index: u32,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    name_player(&p.get())?;
    use rc2k_core::menu::standings::PlayerA;
    let name_id = name_id_of(&p.get())?;
    let source = PlayerA {
        index,
        time: p.get().time,
        group: p.get().class,
        total: p.get().total,
        name_id,
        name: name_of(play, &p.get())?,
        nationality: p.get().nationality,
        car: p.get().car_number,
        entered: p.get().entry,
    };
    with_a(tables, |t| {
        rc2k_core::saves::rankings::fill_a(&mut t.entries[e], &source);
    })?;
    Ok(())
}
