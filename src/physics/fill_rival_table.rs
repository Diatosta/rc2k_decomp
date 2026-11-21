//! The 90 rivals a car's time at a split is ranked against: by default, rival c's time is 12000 +
//! 1000 c, its car c % 23 and its name string 0x5A5 + c; while the race has a standings summary
//! (`rivals` set), its 90 rows' names, cars and times instead. A replay keeps the table its race
//! had (`replay_v2::Extension::rivals`), and plays back with it.
use crate::errors::Refused;
use crate::physics::split_tables_store::SplitTables;
use crate::race::vars_store::RaceVars;
use rc2k_core::physics::splits::{RIVALS, default_rival};
use rc2k_core::race::replay_v2::Rival;
use rc2k_core::saves::name_id::NameId;

/// The race's rivals: the default ones, or the standings summary's rows while it has one.
// Core: `rc2k_core::physics::splits::default_rival`.
pub fn race_rivals(
    vars: &RaceVars,
    tables: &crate::saves::best_times::BestTables,
) -> Result<Vec<Rival>, Refused> {
    let mut rivals: Vec<Rival> = (0..RIVALS as u32)
        .map(|c| {
            let (time, car, name) = default_rival(c);
            Rival {
                name: NameId::String(name).to_file(),
                car,
                time,
            }
        })
        .collect();
    // The standings summary (`RACE_STATE.settings.rivals` is 1 while it is there).
    if vars.rivals.get() != 0 {
        let rows = tables.summary.with(|t| t.rows)?;
        for (to, r) in rivals.iter_mut().zip(rows.iter()) {
            *to = Rival {
                name: r.name_id.to_file(),
                car: r.car,
                time: r.time,
            };
        }
    }
    Ok(rivals)
}

/// The split tables' rivals from `rivals`; a shorter table keeps the defaults past its end.
pub fn fill_rival_table(split_tables: &mut SplitTables, rivals: &[Rival]) {
    for c in 0..RIVALS {
        let r = rivals.get(c).copied().unwrap_or_else(|| {
            let (time, car, name) = default_rival(c as u32);
            Rival {
                name: NameId::String(name).to_file(),
                car,
                time,
            }
        });
        split_tables.rival_names[c] = NameId::from_file(r.name);
        split_tables.rival_cars[c] = r.car;
        split_tables.rival_times[c] = r.time;
    }
}
