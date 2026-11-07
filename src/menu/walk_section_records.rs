//! `menu::walk_section_records` (0x471F0D): the current section's records walked from
//! `index`: a record tagged 0 gets the computer drivers' times (`menu::calc_cpu_times`)
//! and `dword_755944` 0, one tagged 1 the drivers' condition reset
//! (`menu::reset_driver_condition`) and `dword_755944` 1; past each of those and of any
//! other tag but 3 and 4, `index` and `dword_755950` up by one and the players' totals
//! reset (`menu::reset_player_totals`). Tag 4 ends the walk, tag 3 too with
//! `dword_755928` 1; then the drivers' condition reset.
//!
//! The steps are their Rust versions. Refuses what they refuse
//! for any record of the walk, and a walk past 64 records, before anything.
use crate::errors::Refused;
use crate::menu::event_flow::Fault;
use crate::menu::owner::Menus;
use crate::menu::owner::Screens;
use crate::menu::standings::*;
use crate::records::stages_store::RecordRef;
use crate::system::owner::system;
use rc2k_core::menu::section_walk::Step;

/// The record at `index` of the current section.
fn record_at(index: u32) -> Result<RecordRef, crate::errors::BadInput> {
    let table = system().records.stage_tables.championship();
    let section: u32 = SECTION_RVA.get();
    crate::records::find_record::find_record_in(table, index as i32, section)
}

/// The records the walk visits from `index`, with their steps, checked.
fn walk(screens: &Screens) -> Result<Vec<(u32, Step)>, Refused> {
    current_event(screens)?;
    counted_players(screens)?;
    drivers()?;
    let index: u32 = INDEX_RVA.get();
    // Converted: `rc2k_core::menu::section_walk::walk`.
    let fault = Fault::default();
    let walked = rc2k_core::menu::section_walk::walk(
        index,
        |i| fault.ok(record_at(i).map(RecordRef::tag)),
        |i| crate::menu::calc_cpu_times::check(screens, record_at(i)?).map(|_| ()),
    );
    fault.check()?;
    walked
}

/// The Rust version.
pub fn walk_section_records(menus: &Menus) -> Result<(), Refused> {
    let screens = &menus.screens;
    use crate::menu::{reset_driver_condition as rdc, reset_player_totals as rpt};
    let records = walk(screens)?;
    for (_, step) in records {
        if !step.goes_on() {
            if step == Step::Ended {
                screens.event.championship_end.set(1u32);
            }
            break;
        }
        match step {
            Step::CpuTimes => {
                crate::menu::calc_cpu_times::calc_cpu_times(screens)?;
            }
            Step::Condition => {
                rdc::reset_driver_condition()?;
            }
            _ => {}
        }
        if let Some(first) = step.first() {
            screens.event.service_944.set(first);
        }
        INDEX_RVA.set(INDEX_RVA.get().wrapping_add(1));
        screens
            .event
            .stages_done
            .set(screens.event.stages_done.get().wrapping_add(1));
        rpt::reset_player_totals(menus)?;
    }
    rdc::reset_driver_condition()?;
    Ok(())
}
