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
use crate::errors::Refused;
use crate::menu::event_flow::Fault;
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
    let section: u32 = menus.screens.event.section.get();
    crate::records::find_record::find_record_in(
        &system.records().stage_tables,
        table,
        index as i32,
        section,
    )
}

/// The records the walk visits from `index`, with their steps, checked.
fn walk(menus: &Menus, system: &System, race: &Race) -> Result<Vec<(u32, Step)>, Refused> {
    let screens = &menus.screens;
    current_event(screens, system)?;
    counted_players(menus)?;
    drivers(menus)?;
    let index: u32 = menus.screens.event.stage_in_section.get();
    let fault = Fault::default();
    let walked = rc2k_core::menu::section_walk::walk(
        index,
        |i| fault.ok(record_at(menus, system, i).map(|r| r.tag(&system.records().stage_tables))),
        |i| {
            crate::menu::calc_cpu_times::check(menus, system, race, record_at(menus, system, i)?)
                .map(|_| ())
        },
    );
    fault.check()?;
    walked
}

pub fn walk_section_records(menus: &Menus, system: &System, race: &Race) -> Result<(), Refused> {
    let screens = &menus.screens;
    use crate::menu::{reset_driver_condition as rdc, reset_player_totals as rpt};
    let records = walk(menus, system, race)?;
    for (_, step) in records {
        if !step.goes_on() {
            if step == Step::Ended {
                screens.event.championship_end.set(1u32);
            }
            break;
        }
        match step {
            Step::CpuTimes => {
                crate::menu::calc_cpu_times::calc_cpu_times(menus, system, race)?;
            }
            Step::Condition => {
                rdc::reset_driver_condition(menus)?;
            }
            _ => {}
        }
        if let Some(first) = step.first() {
            screens.event.service.spread_first.set(first);
        }
        screens
            .event
            .stage_in_section
            .set(menus.screens.event.stage_in_section.get().wrapping_add(1));
        screens
            .event
            .stages_done
            .set(screens.event.stages_done.get().wrapping_add(1));
        rpt::reset_player_totals(menus)?;
    }
    rdc::reset_driver_condition(menus)?;
    Ok(())
}
