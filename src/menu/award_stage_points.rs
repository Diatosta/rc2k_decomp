//! `menu::award_stage_points`: the stage's championship points. For each class
//! (`event.shown.class`, 0 .. `CLASSES`) the points start at 0x14 (`seconds`); each
//! entry
//! of standings table B in that class, in the table's order, is logged ("Status of
//! driver" and its `status`), gets the points into its `stage` slot of `section` and the points
//! go down by one (not below 0). A computer's entry (`player` 0, its driver found by
//! `menu::find_cpu_team` from the entry's `first`) keeps them in the driver's results when the
//! driver is still in the event, a player's entry (`player` the player + 1) in the player's
//! `stage_points` when the player takes part; otherwise both are 0.
//! Then each entry's total of its six stage points (the negative ones left out) goes into its
//! own total slot and into its driver's or player's total.
//!
//! It logs through `logfile::log_write` and `log_int` (level 1). Refuses no table, one without its
//! end entry, a section past 5, a player past 4 or missing,
//! before anything; a driver not found is the search's fatal error, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::drivers_store::IN_EVENT;
use crate::menu::owner::Menus;
use crate::menu::standings::*;

const STATUS_TEXT: &std::ffi::CStr = c"Status of driver";
/// The driver (computer, by `find_cpu_team`) or player of an entry, and the field its
/// stage points go to, and whether it keeps them.
enum Owner<'a> {
    Driver(usize),
    Player(PlayerCell<'a>),
}

fn owner<'a>(
    menus: &'a Menus,
    e: &rc2k_core::saves::rankings::HistoryEntry,
) -> Result<Owner<'a>, Refused> {
    match e.player {
        0 => Ok(Owner::Driver(crate::menu::find_cpu_team::find_cpu_team(
            menus, e.first,
        )?)),
        n if n <= 4 => Ok(Owner::Player(player(menus, n - 1))),
        _ => Err(BadInput("a player past 4 or missing").into()),
    }
}

pub fn award_stage_points(
    menus: &Menus,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    let classes = crate::menu::event_store::CLASSES;
    let section: u32 = screens.event.read().section;
    if section > 5 {
        return Err(BadInput("a section past 5").into());
    }
    let mut all = walked_entries_b(tables)?;
    let mut owners = Vec::with_capacity(all.len());
    for e in &all {
        owners.push(owner(menus, e)?);
    }
    use rc2k_core::menu::points::{Points, kept, total};
    for class in 0..classes {
        let mut points = Points::default();
        for (e, o) in all.iter_mut().zip(&owners) {
            screens.event.write().shown.class = class;
            if e.group != class {
                continue;
            }
            crate::logfile::log_write::log_nul(&STATUS_TEXT, false, 1);
            crate::logfile::log_int::log_int(e.status as i32, true, 1);
            let given = points.take();
            let keeps = match *o {
                Owner::Driver(d) => menus.cars.drivers.read().drivers[d].in_event == IN_EVENT,
                Owner::Player(p) => p.get().entry == 1,
            };
            e.stage[section as usize] = kept(given, keeps);
            match *o {
                Owner::Driver(d) => {
                    *menus.cars.drivers.write().drivers[d].stage(section as usize) =
                        kept(given, keeps);
                }
                Owner::Player(p) => {
                    p.update(|q| q.stage_points[section as usize] = kept(given, keeps));
                }
            }
        }
    }
    for (e, o) in all.iter_mut().zip(&owners) {
        let total = total(std::array::from_fn(|k| e.stage[k]));
        e.stage[6] = total;
        match *o {
            Owner::Driver(d) => *menus.cars.drivers.write().drivers[d].stage_total() = total,
            Owner::Player(p) => p.update(|q| q.stage_points[6] = total),
        }
    }
    with_b(tables, |t| t.entries[..all.len()].copy_from_slice(&all))?;
    Ok(())
}
