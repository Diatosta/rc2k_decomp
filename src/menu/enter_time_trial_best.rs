//! `menu::enter_time_trial_best`: the current player's time put in the tenth of the chosen
//! stage's best times and moved up to its place. the
//! tenth entry's `key` is the player's `time`, `group` its `class`, `nation` the nationality,
//! `name_id` the name pointer, `car` the car number, `player` 1, `mark` 1 (the new entry's
//! mark), `cheats` the cheats, `sealed` the sealed time (`menu::sealed_player_time` for the
//! stage's `event_slot`), `checkpoints` the player's `best_line` and `name` the name's first 31
//! bytes; then `menu::rank_time_trial_best`.
//!
//! Refuses (before anything) a current player past the four or missing, a stage the tables do not
//! give, or a name pointer of 0.

use crate::errors::Refused;
use crate::menu::event_flow::*;
use crate::menu::owner::Menus;
use crate::menu::standings::name_address_of;

pub(super) fn enter_time_trial_best(
    race: &crate::race::owner::Race,
    menus: &Menus,
    system: &crate::system::owner::System,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    use rc2k_core::menu::bests;
    let record = stage_record(menus, system)?;
    stage_times(menus, system, tables)?; // a stage the tables do not give: refused
    let p = current_player(menus)?;
    let name = name_address_of(&p.get())?;
    let slot = record.record(&system.records().stage_tables)?.event_slot();
    let cheats = menus.play.cheats.cheats.get();
    let sealed = crate::menu::sealed_player_time::sealed_player_time(menus, race, slot)?;
    // The logic is in `rc2k_core::saves::entries::BestTime::enter`, then `rank_time_trial_best`.
    use crate::menu::standings::name_of;
    let best = bests::NewBest {
        key: p.get().time,
        player: 1,
        nationality: p.get().nationality,
        name_at: name,
        car: p.get().car_number,
        group: p.get().class,
        name: name_of(&menus.play, &p.get())?,
        entered: None,
        seal: Some(bests::Seal { cheats, sealed }),
        line: Some(p.get().best_line),
    };
    with_stage_times(menus, system, tables, |list| list[9].enter(&best))?;
    crate::menu::rank_time_trial_best::rank_time_trial_best(menus, system, tables)
}
