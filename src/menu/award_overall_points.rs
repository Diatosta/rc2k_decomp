//! `menu::award_overall_points` (0x4A4933): the stage's overall championship points. The
//! points start at 0x14 (`seconds`); each entry of standings table B whose +0x10 is not 0,
//! in the table's order, gets them into its +0x38 + 4 * `section` and they go down by one
//! (not below 0); a computer's driver (+4 0, `menu::find_cpu_team` by its +0) also into
//! its +0x5C + 4 * `section`, a player (+4 the player + 1) into its +0xB4 + 4 * `section`.
//! Then each entry's total of its six (+0x38 .., the negative ones left out) into its
//! +0x50 and its driver's +0x74 or player's +0xCC.
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

pub fn award_overall_points(menus: &Menus) -> Result<(), Refused> {
    let screens = &menus.screens;
    {
        let section: u32 = SECTION_RVA.get();
        if section > 5 {
            return Err(BadInput("a section past 5").into());
        }
        let mut all = walked_entries_b()?;
        let mut owners = Vec::with_capacity(all.len());
        for e in &all {
            owners.push(owner(menus, e)?);
        }
        // Converted: `rc2k_core::menu::points` (`Points`, `total`).
        use rc2k_core::menu::points::{Points, total};
        let mut points = Points::default();
        screens.draw_work.seconds.set(points.0);
        for (e, &o) in all.iter_mut().zip(&owners) {
            if e.group == 0 {
                continue;
            }
            let given = points.take();
            screens.draw_work.seconds.set(points.0);
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
        with_b(|t| t.entries[..all.len()].copy_from_slice(&all))?;
        Ok(())
    }
}
