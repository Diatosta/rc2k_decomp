//! `menu::award_overall_points`: the stage's overall championship points. The
//! points start at 20 (`seconds`); each entry of standings table B that has a group, in the
//! table's order, gets them into its `overall` slot of `section` and they go down by one (not
//! below 0); a computer's entry (`player` 0, its driver found by `menu::find_cpu_team` from the
//! entry's `first`) also into the driver's overall points of that section, a player's entry
//! (`player` the player + 1) into the player's `overall_points` of that section.
//! Then each entry's total of its six (the negative ones left out) goes into its own total slot
//! and into its driver's or player's total.
//!
//! Refuses no table, one without its end
//! entry, a section past 5, a player past 4 or missing, before anything; a driver not found is the
//! search's fatal error, before anything.
use crate::errors::{BadInput, Refused};
use crate::menu::drivers_store::Driver;
use crate::menu::owner::Menus;
use crate::menu::standings::*;

/// An entry's driver (computer) or player.
#[derive(Clone, Copy)]
enum Owner<'a> {
    Driver(&'a Driver),
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

pub fn award_overall_points(
    menus: &Menus,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    {
        let section: u32 = screens.event.section.get();
        if section > 5 {
            return Err(BadInput("a section past 5").into());
        }
        let mut all = walked_entries_b(tables)?;
        let mut owners = Vec::with_capacity(all.len());
        for e in &all {
            owners.push(owner(menus, e)?);
        }
        use rc2k_core::menu::points::{Points, total};
        let mut points = Points::default();
        for (e, &o) in all.iter_mut().zip(&owners) {
            if e.group == 0 {
                continue;
            }
            let given = points.take();
            e.overall[section as usize] = given;
            match o {
                Owner::Driver(d) => d.overall(section as usize).set(given),
                Owner::Player(p) => p.update(|q| q.overall_points[section as usize] = given),
            }
        }
        for (e, &o) in all.iter_mut().zip(&owners) {
            let total = total(std::array::from_fn(|k| e.overall[k]));
            e.overall[6] = total;
            match o {
                Owner::Driver(d) => d.overall_total().set(total),
                Owner::Player(p) => p.update(|q| q.overall_points[6] = total),
            }
        }
        with_b(tables, |t| t.entries[..all.len()].copy_from_slice(&all))?;
        Ok(())
    }
}
