//! `menu::walk_section_records`: the current section's records walked from
//! `index`: a record tagged 0 gets the computer drivers' times (`menu::calc_cpu_times`)
//! and `event.service.spread_first` 0, one tagged 1 the drivers' condition reset
//! (`menu::reset_driver_condition`) and `event.service.spread_first` 1; past each of those and of
//! any
//! other tag but 3 and 4, `index` and `event.stages_done` up by one and the players' totals
//! reset (`menu::reset_player_totals`). Tag 4 ends the walk, tag 3 too with
//! `event.championship_end` 1; then the drivers' condition reset.
//!
//! Refuses what they refuse
//! for any record of the walk, and a walk past 64 records, before anything.
use crate::errors::Fault;
use crate::errors::Refused;
use crate::menu::owner::Menus;
use crate::menu::standings::*;
use crate::race::owner::Race;
use crate::records::stages_store::RecordRef;
use crate::system::owner::System;
use rc2k_core::menu::section_walk::Step;

/// The record at `index` of the current section.
fn record_at(
    menus: &Menus,
    system: &System,
    index: u32,
) -> Result<RecordRef, crate::errors::BadInput> {
    let table = system.records().stage_tables.championship();
    let section: u32 = menus.screens.event.read().section;
    crate::records::find_record::find_record_in(
        &system.records().stage_tables,
        table,
        index as i32,
        section,
    )
}

/// The records the walk visits from `index`, with their steps, checked.
fn walk(
    menus: &Menus,
    system: &System,
    race: &Race,
    tables: &crate::saves::best_times::BestTables,
) -> Result<Vec<(u32, Step)>, Refused> {
    let screens = &menus.screens;
    current_event(screens, system, tables)?;
    counted_players(&menus.screens, &menus.play)?;
    drivers(menus)?;
    let index: u32 = menus.screens.event.read().stage_in_section;
    let fault = Fault::default();
    let walked = rc2k_core::menu::section_walk::walk(
        index,
        |i| fault.ok(record_at(menus, system, i).map(|r| r.tag(&system.records().stage_tables))),
        |i| {
            crate::menu::calc_cpu_times::check(
                &menus.cars,
                screens,
                system,
                race,
                record_at(menus, system, i)?,
            )
            .map(|_| ())
        },
    );
    fault.check()?;
    walked
}

pub(super) fn walk_section_records(
    menus: &Menus,
    system: &System,
    race: &Race,
    tables: &crate::saves::best_times::BestTables,
) -> Result<(), Refused> {
    let screens = &menus.screens;
    use crate::menu::{reset_driver_condition as rdc, reset_player_totals as rpt};
    let records = walk(menus, system, race, tables)?;
    for (_, step) in records {
        if !step.goes_on() {
            if step == Step::Ended {
                screens.event.write().championship_end = 1u32;
            }
            break;
        }
        match step {
            Step::CpuTimes => {
                crate::menu::calc_cpu_times::calc_cpu_times(
                    &menus.cars,
                    screens,
                    system,
                    race,
                    tables,
                )?;
            }
            Step::Condition => {
                rdc::reset_driver_condition(menus)?;
            }
            _ => {}
        }
        if let Some(first) = step.first() {
            screens.event.write().service.spread_first = first;
        }
        screens.event.write().stage_in_section =
            menus.screens.event.read().stage_in_section.wrapping_add(1);
        screens.event.write().stages_done = screens.event.read().stages_done.wrapping_add(1);
        rpt::reset_player_totals(menus)?;
    }
    rdc::reset_driver_condition(menus)?;
    Ok(())
}
